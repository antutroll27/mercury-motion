# Mercury-Motion Templates

Ready-to-use animation templates. Pick one, customize it, render it.

## How to Use

1. Copy a template to your project
2. Edit the text, colors, and timing
3. Render: `mmot render template.mmot.json --output video.mp4`

## Categories

### Social Media
| Template | Size | Duration | Description |
|----------|------|----------|-------------|
| [Instagram Post](social/instagram-post.mmot.json) | 1080x1080 | 3s | Headline + subtitle + handle |
| [Instagram Story](social/instagram-story.mmot.json) | 1080x1920 | 4s | Swipe up CTA |
| [YouTube Intro](social/youtube-intro.mmot.json) | 1920x1080 | 3s | Channel name reveal |
| [TikTok Overlay](social/tiktok-overlay.mmot.json) | 1080x1920 | 2s | Username + caption overlay |

### Business
| Template | Size | Duration | Description |
|----------|------|----------|-------------|
| [Presentation Title](business/presentation-title.mmot.json) | 1920x1080 | 3s | Company + title + date |
| [Lower Third](business/lower-third.mmot.json) | 1920x1080 | 3s | Name + title bar |
| [Stats Counter](business/stats-counter.mmot.json) | 1920x1080 | 3s | 3 animated statistics |

### Creative
| Template | Size | Duration | Description |
|----------|------|----------|-------------|
| [Neon Text](creative/neon-text.mmot.json) | 1920x1080 | 2s | Glitch/neon text effect |
| [Particle Explosion](creative/particle-explosion.mmot.json) | 1080x1080 | 2s | Burst of glowing particles |
| [Logo Placeholder](creative/logo-placeholder.mmot.json) | 1920x1080 | 2.5s | Logo reveal with effects |

### UI Elements
| Template | Size | Duration | Description |
|----------|------|----------|-------------|
| [Loading Dots](ui/loading-dots.mmot.json) | 200x60 | 1s | Bouncing dots loader |
| [Notification Badge](ui/notification-badge.mmot.json) | 80x80 | 0.5s | Spring bounce badge |
| [Progress Bar](ui/progress-bar.mmot.json) | 400x40 | 2s | Animated fill bar |
| [Button Pulse](ui/button-pulse.mmot.json) | 240x60 | 1.5s | Pulsing CTA button |

### Transitions
| Template | Size | Duration | Description |
|----------|------|----------|-------------|
| [Crossfade](transitions/fade.mmot.json) | 1920x1080 | 1s | Smooth opacity crossfade |
| [Wipe Left](transitions/wipe-left.mmot.json) | 1920x1080 | 1s | Horizontal wipe |

## Customization Tips
- Change `"text"` values to your own copy
- Change `"color"` / `"fill"` values to your brand colors
- Adjust `"duration"` in meta for longer/shorter
- Adjust `"fps"` — use 15 for GIFs, 30 for video
