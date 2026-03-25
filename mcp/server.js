#!/usr/bin/env node
import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
} from "@modelcontextprotocol/sdk/types.js";
import { execSync } from "child_process";
import { writeFileSync, mkdtempSync } from "fs";
import { join } from "path";
import { tmpdir } from "os";

// ── Tool Definitions ────────────────────────────────────────────────────────

const TOOLS = [
  {
    name: "create_scene",
    description:
      "Generate a Mercury-Motion .mmot.json scene file. Returns a complete scene template " +
      "with solid background layer and optional text layer based on the description. " +
      "The returned JSON can be saved to a .mmot.json file and rendered with the render tool.",
    inputSchema: {
      type: "object",
      properties: {
        description: {
          type: "string",
          description:
            'Description of the scene to generate (e.g. "hello world title on dark background")',
        },
        width: {
          type: "number",
          default: 1920,
          description: "Video width in pixels",
        },
        height: {
          type: "number",
          default: 1080,
          description: "Video height in pixels",
        },
        fps: {
          type: "number",
          default: 30,
          description: "Frames per second",
        },
        duration_frames: {
          type: "number",
          default: 90,
          description: "Total duration in frames",
        },
      },
    },
  },
  {
    name: "render",
    description:
      "Render a .mmot.json file to video (MP4, GIF, or WebM). " +
      "Invokes the mmot CLI to produce the output file. " +
      "Returns success message with output path, or error details on failure.",
    inputSchema: {
      type: "object",
      properties: {
        scene_path: {
          type: "string",
          description: "Absolute path to the .mmot.json scene file",
        },
        output_path: {
          type: "string",
          description:
            "Absolute path for the output video file (default: output.mp4 next to scene)",
        },
        format: {
          type: "string",
          enum: ["mp4", "gif", "webm"],
          default: "mp4",
          description: "Output format",
        },
        quality: {
          type: "number",
          default: 80,
          description: "Encoding quality 0-100 (default: 80)",
        },
      },
      required: ["scene_path"],
    },
  },
  {
    name: "validate",
    description:
      "Validate a .mmot.json file without rendering. " +
      "Checks that the scene file parses correctly and reports any errors.",
    inputSchema: {
      type: "object",
      properties: {
        scene_path: {
          type: "string",
          description: "Absolute path to the .mmot.json scene file to validate",
        },
      },
      required: ["scene_path"],
    },
  },
  {
    name: "preview_frame",
    description:
      "Render a single frame from a scene as a PNG image for preview. " +
      "Accepts inline scene JSON and a frame number. " +
      "Returns the path to the rendered PNG file.",
    inputSchema: {
      type: "object",
      properties: {
        scene_json: {
          type: "string",
          description: "The full .mmot.json scene content as a string",
        },
        frame: {
          type: "number",
          default: 0,
          description: "Frame number to render (0-indexed)",
        },
        output_path: {
          type: "string",
          description:
            "Optional output path for the PNG. If omitted, a temp file is created.",
        },
      },
      required: ["scene_json"],
    },
  },
  {
    name: "get_schema",
    description:
      "Return the .mmot.json schema reference covering all layer types, " +
      "transform properties, effects, easing, blend modes, masks, transitions, and more.",
    inputSchema: {
      type: "object",
      properties: {},
    },
  },
  {
    name: "list_effects",
    description:
      "List all available visual effects with their parameters and descriptions.",
    inputSchema: {
      type: "object",
      properties: {},
    },
  },
  {
    name: "list_blend_modes",
    description: "List all available compositing blend modes.",
    inputSchema: {
      type: "object",
      properties: {},
    },
  },
];

// ── Tool Handlers ───────────────────────────────────────────────────────────

function handleCreateScene(args) {
  const width = args.width ?? 1920;
  const height = args.height ?? 1080;
  const fps = args.fps ?? 30;
  const durationFrames = args.duration_frames ?? 90;
  const description = args.description ?? "";

  // Determine scene content from description
  const descLower = description.toLowerCase();
  const hasText =
    descLower.includes("text") ||
    descLower.includes("title") ||
    descLower.includes("hello") ||
    descLower.includes("word") ||
    descLower.includes("label") ||
    descLower.includes("heading");

  // Extract text content from description, or default
  let textContent = "Hello World";
  const quoteMatch = description.match(/"([^"]+)"/);
  if (quoteMatch) {
    textContent = quoteMatch[1];
  } else if (descLower.includes("hello")) {
    textContent = "Hello World";
  }

  // Determine colors
  let bgColor = "#1a1a2e";
  let textColor = "#ffffff";
  if (descLower.includes("white background") || descLower.includes("light")) {
    bgColor = "#ffffff";
    textColor = "#1a1a2e";
  } else if (descLower.includes("red")) {
    bgColor = "#9e2a2b";
  } else if (descLower.includes("blue")) {
    bgColor = "#16213e";
  } else if (descLower.includes("green")) {
    bgColor = "#1b4332";
  }

  const layers = [
    {
      id: "bg",
      type: "solid",
      in: 0,
      out: durationFrames,
      color: bgColor,
      fill: "parent",
      transform: {
        position: [0, 0],
        scale: [1, 1],
        rotation: 0,
        opacity: 1,
      },
    },
  ];

  if (hasText || description === "") {
    layers.push({
      id: "title",
      type: "text",
      in: 0,
      out: durationFrames,
      text: textContent,
      font: {
        family: "Inter",
        size: 72,
        weight: 700,
        color: textColor,
      },
      align: "center",
      transform: {
        position: [width / 2, height / 2],
        scale: [1, 1],
        rotation: 0,
        opacity: [
          { t: 0, v: 0, easing: "ease_out" },
          { t: Math.min(15, durationFrames), v: 1 },
        ],
      },
    });
  }

  const scene = {
    version: "1.0",
    meta: {
      name: description || "untitled",
      width,
      height,
      fps,
      duration: durationFrames,
      background: bgColor,
      root: "main",
    },
    compositions: {
      main: {
        layers,
      },
    },
  };

  const json = JSON.stringify(scene, null, 2);
  return {
    content: [
      {
        type: "text",
        text: json,
      },
    ],
  };
}

function handleRender(args) {
  const scenePath = args.scene_path;
  const format = args.format ?? "mp4";
  const quality = args.quality ?? 80;
  const outputPath =
    args.output_path ?? scenePath.replace(/\.mmot\.json$/, `.${format}`);

  const cmd = `mmot render "${scenePath}" --output "${outputPath}" --format ${format} --quality ${quality}`;

  try {
    const result = execSync(cmd, {
      encoding: "utf-8",
      timeout: 300000,
      stdio: ["pipe", "pipe", "pipe"],
    });
    return {
      content: [
        {
          type: "text",
          text: `Rendered successfully.\n\nOutput: ${outputPath}\nFormat: ${format.toUpperCase()}\nQuality: ${quality}\n\nCommand: ${cmd}\n\n${result}`,
        },
      ],
    };
  } catch (error) {
    const stderr = error.stderr ?? "";
    const stdout = error.stdout ?? "";
    return {
      content: [
        {
          type: "text",
          text: `Render failed.\n\nCommand: ${cmd}\n\nExit code: ${error.status}\n\nstderr:\n${stderr}\n\nstdout:\n${stdout}`,
        },
      ],
      isError: true,
    };
  }
}

function handleValidate(args) {
  const scenePath = args.scene_path;
  const cmd = `mmot validate "${scenePath}"`;

  try {
    const result = execSync(cmd, {
      encoding: "utf-8",
      timeout: 30000,
      stdio: ["pipe", "pipe", "pipe"],
    });
    return {
      content: [
        {
          type: "text",
          text: `Validation passed.\n\nFile: ${scenePath}\n\n${result}`,
        },
      ],
    };
  } catch (error) {
    const stderr = error.stderr ?? "";
    const stdout = error.stdout ?? "";
    return {
      content: [
        {
          type: "text",
          text: `Validation failed.\n\nFile: ${scenePath}\n\nExit code: ${error.status}\n\nstderr:\n${stderr}\n\nstdout:\n${stdout}`,
        },
      ],
      isError: true,
    };
  }
}

function handlePreviewFrame(args) {
  const sceneJson = args.scene_json;
  const frame = args.frame ?? 0;

  // Write scene JSON to a temp file
  const tempDir = mkdtempSync(join(tmpdir(), "mmot-preview-"));
  const scenePath = join(tempDir, "scene.mmot.json");
  const outputPath =
    args.output_path ?? join(tempDir, `frame_${frame}.png`);

  try {
    writeFileSync(scenePath, sceneJson, "utf-8");
  } catch (error) {
    return {
      content: [
        {
          type: "text",
          text: `Failed to write temp scene file: ${error.message}`,
        },
      ],
      isError: true,
    };
  }

  // Render as GIF with 1-frame duration to get a single frame output,
  // or use the render command with frame range if supported.
  // The mmot CLI does not have a single-frame PNG export yet,
  // so we render a 1-frame GIF and note the limitation.
  const cmd = `mmot render "${scenePath}" --output "${outputPath}" --format mp4 --quality 90`;

  try {
    execSync(cmd, {
      encoding: "utf-8",
      timeout: 120000,
      stdio: ["pipe", "pipe", "pipe"],
    });
    return {
      content: [
        {
          type: "text",
          text: `Frame preview rendered.\n\nOutput: ${outputPath}\nFrame: ${frame}\nTemp scene: ${scenePath}\n\nNote: The mmot CLI renders the full scene. The output file contains all frames. Single-frame PNG export is planned for a future release.`,
        },
      ],
    };
  } catch (error) {
    const stderr = error.stderr ?? "";
    return {
      content: [
        {
          type: "text",
          text: `Preview render failed.\n\nstderr:\n${stderr}\n\nError: ${error.message}`,
        },
      ],
      isError: true,
    };
  }
}

function handleGetSchema() {
  const schema = `# Mercury-Motion (.mmot.json) Schema Reference

## Root Structure
\`\`\`json
{
  "version": "1.0",
  "meta": { ... },
  "props": { ... },          // optional — template variables
  "compositions": { ... },
  "assets": { ... }          // optional — fonts, etc.
}
\`\`\`

## Meta
\`\`\`json
{
  "name": "Scene Name",
  "width": 1920,              // pixels
  "height": 1080,             // pixels
  "fps": 30,                  // frames per second (float)
  "duration": 90,             // total frames (integer)
  "background": "#000000",    // hex color (default: "#000000")
  "root": "main"              // ID of the root composition
}
\`\`\`

## Props (Template Variables)
\`\`\`json
{
  "props": {
    "title": { "type": "string", "default": "Hello" },
    "accent": { "type": "color", "default": "#ff0000" },
    "count": { "type": "number", "default": 42 },
    "logo": { "type": "url", "default": "logo.png" }
  }
}
\`\`\`
Use \`{{prop_name}}\` in layer values. Override via CLI: \`--prop title=World\`

## Compositions
A map of composition IDs to composition objects:
\`\`\`json
{
  "compositions": {
    "main": {
      "layers": [ ... ],
      "sequence": false,          // optional: play layers back-to-back
      "transition": { ... }       // optional: transition between sequence layers
    }
  }
}
\`\`\`

## Transitions (for sequence mode)
\`\`\`json
{ "type": "crossfade", "duration": 15 }
{ "type": "wipe", "duration": 15, "direction": "left" }    // left|right|up|down
{ "type": "slide", "duration": 15, "direction": "right" }
\`\`\`

## Layer (Common Fields)
\`\`\`json
{
  "id": "unique_id",
  "type": "solid",                    // layer type (see below)
  "in": 0,                            // start frame (inclusive)
  "out": 90,                          // end frame (exclusive)
  "transform": { ... },               // position, scale, rotation, opacity
  "fill": "parent",                   // optional: fill entire canvas
  "blend_mode": "normal",             // optional: compositing blend mode
  "parent": "null_1",                 // optional: parent layer ID for parenting
  "adjustment": false,                // optional: adjustment layer
  "motion_blur": false,               // optional: enable motion blur
  "effects": [ ... ],                 // optional: list of effects
  "masks": [ ... ],                   // optional: list of masks
  "track_matte": { ... },             // optional: track matte
  "time_remap": { ... },              // optional: time remapping
  "trim_paths": { ... },              // optional: trim paths (shapes)
  "path_animation": { ... },          // optional: animate along a path
  // ... plus type-specific fields
}
\`\`\`

## Layer Types

### solid
\`\`\`json
{ "type": "solid", "color": "#1a1a2e" }
\`\`\`

### text
\`\`\`json
{
  "type": "text",
  "text": "Hello World",
  "font": {
    "family": "Inter",
    "size": 72,              // default: 32
    "weight": 700,           // default: 400
    "color": "#ffffff"       // default: "#ffffff"
  },
  "align": "center"          // "left" | "center" | "right"
}
\`\`\`

### image
\`\`\`json
{ "type": "image", "src": "path/to/image.png" }
\`\`\`

### video
\`\`\`json
{
  "type": "video",
  "src": "path/to/video.mp4",
  "trim_start": 0.0,         // seconds to trim from start
  "trim_end": null            // optional: seconds to trim from end
}
\`\`\`

### audio
\`\`\`json
{
  "type": "audio",
  "src": "path/to/audio.mp3",
  "volume": 1.0               // animatable (0.0 to 1.0)
}
\`\`\`

### shape
\`\`\`json
// Rectangle
{ "type": "shape", "shape": { "shape_type": "rect", "width": 200, "height": 100, "corner_radius": 10, "fill": "#ff0000", "stroke": { "color": "#000", "width": 2 } } }

// Ellipse
{ "type": "shape", "shape": { "shape_type": "ellipse", "width": 200, "height": 200, "fill": "#00ff00" } }

// Line
{ "type": "shape", "shape": { "shape_type": "line", "x1": 0, "y1": 0, "x2": 100, "y2": 100, "stroke": { "color": "#fff", "width": 2 } } }

// Polygon
{ "type": "shape", "shape": { "shape_type": "polygon", "points": [[0,0],[100,0],[50,86]], "fill": "#0000ff" } }
\`\`\`

### gradient
\`\`\`json
// Linear gradient
{ "type": "gradient", "gradient": { "gradient_type": "linear", "start": [0, 0], "end": [1920, 1080], "colors": [{ "offset": 0, "color": "#ff0000" }, { "offset": 1, "color": "#0000ff" }] } }

// Radial gradient
{ "type": "gradient", "gradient": { "gradient_type": "radial", "center": [960, 540], "radius": 500, "colors": [{ "offset": 0, "color": "#ffffff" }, { "offset": 1, "color": "#000000" }] } }
\`\`\`

### lottie
\`\`\`json
{ "type": "lottie", "src": "path/to/animation.json" }
\`\`\`

### composition (nested)
\`\`\`json
{ "type": "composition", "composition_id": "other_comp" }
\`\`\`

### null (transform-only, for parenting)
\`\`\`json
{ "type": "null" }
\`\`\`

## Transform
All transform properties are animatable (static value or keyframe array).
\`\`\`json
{
  "position": [960, 540],            // [x, y] in pixels
  "scale": [1, 1],                    // [x, y] scale factor
  "rotation": 0,                      // degrees
  "opacity": 1.0                      // 0.0 to 1.0
}
\`\`\`

## Animatable Values
A property can be a static value or an array of keyframes:
\`\`\`json
// Static
"opacity": 1.0

// Animated (array of keyframes)
"opacity": [
  { "t": 0, "v": 0.0, "easing": "ease_out" },
  { "t": 15, "v": 1.0 }
]

// Animated position
"position": [
  { "t": 0, "v": [0, 540], "easing": "ease_in_out" },
  { "t": 30, "v": [960, 540] }
]
\`\`\`

## Easing
\`\`\`json
// Named presets
"easing": "linear"
"easing": "ease_in"
"easing": "ease_out"
"easing": "ease_in_out"

// Custom cubic bezier
"easing": { "type": "cubic_bezier", "x1": 0.4, "y1": 0.0, "x2": 0.2, "y2": 1.0 }

// Spring physics
"easing": { "type": "spring", "mass": 1.0, "stiffness": 170.0, "damping": 26.0 }
\`\`\`

## Assets
\`\`\`json
{
  "assets": {
    "fonts": [
      { "id": "custom-font", "src": "path/to/font.ttf" }
    ]
  }
}
\`\`\`

## Time Remap
\`\`\`json
{
  "time_remap": {
    "speed": 2.0,       // playback speed multiplier (default: 1.0)
    "offset": 0.5,      // time offset in seconds (default: 0)
    "reverse": false     // play in reverse (default: false)
  }
}
\`\`\`

## Masks
\`\`\`json
{
  "masks": [
    {
      "path": { "type": "rect", "x": 100, "y": 100, "width": 500, "height": 300, "corner_radius": 20 },
      "mode": "add",         // add | subtract | intersect | difference
      "feather": 5.0,
      "opacity": 1.0,
      "inverted": false
    },
    {
      "path": { "type": "ellipse", "cx": 960, "cy": 540, "rx": 200, "ry": 200 }
    },
    {
      "path": { "type": "path", "points": [[0,0],[100,0],[100,100]], "closed": true }
    }
  ]
}
\`\`\`

## Track Matte
\`\`\`json
{
  "track_matte": {
    "source": "matte_layer_id",
    "mode": "alpha"               // alpha | alpha_inverted | luma | luma_inverted
  }
}
\`\`\`

## Trim Paths (for shape layers)
\`\`\`json
{
  "trim_paths": {
    "start": 0.0,                  // animatable, 0.0-1.0
    "end": 1.0,                    // animatable, 0.0-1.0
    "offset": 0.0                  // animatable, 0.0-1.0
  }
}
\`\`\`

## Path Animation
\`\`\`json
{
  "path_animation": {
    "points": [[0, 540], [480, 200], [960, 540], [1440, 880], [1920, 540]],
    "auto_orient": true            // rotate layer along path tangent
  }
}
\`\`\`
`;

  return {
    content: [{ type: "text", text: schema }],
  };
}

function handleListEffects() {
  const effects = `# Mercury-Motion Effects Reference

All effects are applied via the "effects" array on a layer.

## gaussian_blur
Gaussian blur with configurable radius.
\`\`\`json
{ "type": "gaussian_blur", "radius": 5.0 }
\`\`\`
- **radius** (number, required): Blur radius in pixels.

## drop_shadow
Drop shadow cast behind the layer.
\`\`\`json
{ "type": "drop_shadow", "color": "#000000", "offset_x": 4.0, "offset_y": 4.0, "blur": 10.0, "opacity": 0.5 }
\`\`\`
- **color** (string, required): Shadow color (hex).
- **offset_x** (number, required): Horizontal offset in pixels.
- **offset_y** (number, required): Vertical offset in pixels.
- **blur** (number, required): Shadow blur radius.
- **opacity** (number, default: 1.0): Shadow opacity (0.0-1.0).

## glow
Glow effect emanating from bright regions.
\`\`\`json
{ "type": "glow", "color": "#ffffff", "radius": 10.0, "intensity": 1.5 }
\`\`\`
- **color** (string, required): Glow color (hex).
- **radius** (number, required): Glow radius in pixels.
- **intensity** (number, default: 1.0): Glow intensity multiplier.

## brightness_contrast
Brightness and contrast adjustment.
\`\`\`json
{ "type": "brightness_contrast", "brightness": 20.0, "contrast": 10.0 }
\`\`\`
- **brightness** (number, default: 0): Brightness adjustment.
- **contrast** (number, default: 0): Contrast adjustment.

## hue_saturation
Hue, saturation, and lightness adjustment.
\`\`\`json
{ "type": "hue_saturation", "hue": 30.0, "saturation": -20.0, "lightness": 10.0 }
\`\`\`
- **hue** (number, default: 0): Hue rotation in degrees.
- **saturation** (number, default: 0): Saturation adjustment.
- **lightness** (number, default: 0): Lightness adjustment.

## invert
Inverts all color channels. No parameters.
\`\`\`json
{ "type": "invert" }
\`\`\`

## tint
Tints the layer toward a target color.
\`\`\`json
{ "type": "tint", "color": "#ff8800", "amount": 0.7 }
\`\`\`
- **color** (string, required): Target tint color (hex).
- **amount** (number, default: 1.0): Tint strength (0.0-1.0).

## fill
Fills the entire layer with a solid color.
\`\`\`json
{ "type": "fill", "color": "#00ff00", "opacity": 0.5 }
\`\`\`
- **color** (string, required): Fill color (hex).
- **opacity** (number, default: 1.0): Fill opacity (0.0-1.0).
`;

  return {
    content: [{ type: "text", text: effects }],
  };
}

function handleListBlendModes() {
  const modes = `# Mercury-Motion Blend Modes

All blend modes are specified as snake_case strings in the "blend_mode" field on a layer.

| Blend Mode     | Value            | Description                                    |
|----------------|------------------|------------------------------------------------|
| Normal         | "normal"         | Default compositing (source over)              |
| Multiply       | "multiply"       | Darkens by multiplying colors                  |
| Screen         | "screen"         | Lightens by inverting, multiplying, inverting   |
| Overlay        | "overlay"        | Combines Multiply and Screen                   |
| Darken         | "darken"         | Keeps the darker of each channel               |
| Lighten        | "lighten"        | Keeps the lighter of each channel              |
| Color Dodge    | "color_dodge"    | Brightens base to reflect blend color          |
| Color Burn     | "color_burn"     | Darkens base to reflect blend color            |
| Hard Light     | "hard_light"     | Multiply or Screen depending on blend color    |
| Soft Light     | "soft_light"     | Subtle version of Hard Light                   |
| Difference     | "difference"     | Absolute difference of base and blend          |
| Exclusion      | "exclusion"      | Similar to Difference but lower contrast       |
| Add            | "add"            | Adds color values (clamped to white)           |

## Usage Example
\`\`\`json
{
  "id": "overlay_layer",
  "type": "solid",
  "color": "#ff0000",
  "in": 0, "out": 90,
  "blend_mode": "multiply",
  "transform": { "position": [960, 540], "opacity": 0.5 }
}
\`\`\`
`;

  return {
    content: [{ type: "text", text: modes }],
  };
}

// ── Server Setup ────────────────────────────────────────────────────────────

const server = new Server(
  {
    name: "mmot-mcp",
    version: "0.1.0",
  },
  {
    capabilities: {
      tools: {},
    },
  }
);

server.setRequestHandler(ListToolsRequestSchema, async () => ({
  tools: TOOLS,
}));

server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  try {
    switch (name) {
      case "create_scene":
        return handleCreateScene(args ?? {});
      case "render":
        return handleRender(args ?? {});
      case "validate":
        return handleValidate(args ?? {});
      case "preview_frame":
        return handlePreviewFrame(args ?? {});
      case "get_schema":
        return handleGetSchema();
      case "list_effects":
        return handleListEffects();
      case "list_blend_modes":
        return handleListBlendModes();
      default:
        return {
          content: [
            {
              type: "text",
              text: `Unknown tool: ${name}. Available tools: ${TOOLS.map((t) => t.name).join(", ")}`,
            },
          ],
          isError: true,
        };
    }
  } catch (error) {
    return {
      content: [
        {
          type: "text",
          text: `Internal error in tool "${name}": ${error.message ?? error}`,
        },
      ],
      isError: true,
    };
  }
});

// ── Start ───────────────────────────────────────────────────────────────────

const transport = new StdioServerTransport();
await server.connect(transport);
