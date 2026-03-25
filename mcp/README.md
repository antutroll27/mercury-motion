# Mercury-Motion MCP Server

An MCP (Model Context Protocol) server that wraps the `mmot` CLI, allowing AI assistants to create and render Mercury-Motion animations programmatically.

## Installation

```bash
cd mcp
npm install
```

Requires Node.js 18+ and the `mmot` CLI binary on your PATH.

## Integration with Claude Code

The repository root contains a `.mcp.json` file that configures the server automatically. If you need to configure it manually, add this to your Claude Code MCP config:

```json
{
  "mcpServers": {
    "mmot": {
      "command": "node",
      "args": ["mcp/server.js"],
      "cwd": "/path/to/ReMotion"
    }
  }
}
```

## Available Tools

### create_scene

Generate a `.mmot.json` scene file template from a description.

**Parameters:**
| Name | Type | Default | Description |
|------|------|---------|-------------|
| `description` | string | `""` | Description of the scene (e.g. `"hello world title on dark background"`) |
| `width` | number | `1920` | Video width in pixels |
| `height` | number | `1080` | Video height in pixels |
| `fps` | number | `30` | Frames per second |
| `duration_frames` | number | `90` | Total duration in frames |

**Returns:** Complete `.mmot.json` content as a JSON string.

### render

Render a `.mmot.json` file to video.

**Parameters:**
| Name | Type | Default | Description |
|------|------|---------|-------------|
| `scene_path` | string | *(required)* | Absolute path to the `.mmot.json` file |
| `output_path` | string | auto | Output file path (defaults to scene path with format extension) |
| `format` | `"mp4"` \| `"gif"` \| `"webm"` | `"mp4"` | Output format |
| `quality` | number | `80` | Encoding quality (0-100) |

**Returns:** Success message with output path, or error details.

### validate

Validate a `.mmot.json` file without rendering.

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `scene_path` | string | *(required)* Absolute path to the `.mmot.json` file |

**Returns:** Validation result (pass or fail with error details).

### preview_frame

Render a single frame from inline scene JSON.

**Parameters:**
| Name | Type | Default | Description |
|------|------|---------|-------------|
| `scene_json` | string | *(required)* | Full `.mmot.json` content as a string |
| `frame` | number | `0` | Frame number to render |
| `output_path` | string | auto | Output PNG path (defaults to temp file) |

**Returns:** Path to the rendered output file.

### get_schema

Return the complete `.mmot.json` schema reference covering all layer types, transform properties, effects, easing curves, blend modes, masks, transitions, and more.

**Parameters:** None.

### list_effects

List all available visual effects with their parameters and descriptions.

**Parameters:** None.

### list_blend_modes

List all available compositing blend modes.

**Parameters:** None.

## Example Usage

A typical workflow for an AI assistant:

1. Call `get_schema` to understand the `.mmot.json` format
2. Call `create_scene` with a description to get a starter template
3. Modify the JSON as needed (add layers, animations, effects)
4. Save to a `.mmot.json` file
5. Call `validate` to check for errors
6. Call `render` to produce the final video

## Architecture

The server uses stdio transport and communicates via the Model Context Protocol. It wraps the `mmot` CLI binary for rendering and validation, and generates scene templates directly in JavaScript.
