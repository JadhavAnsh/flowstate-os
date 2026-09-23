# Design System Inspired by VoiceOS

Reference: [voiceos.com](https://www.voiceos.com/) — AI voice assistant marketing site (Mac-native aesthetic, glass UI, sky/blue atmosphere).

## 1. Visual Theme & Atmosphere

VoiceOS reads as **bright Apple-ecosystem SaaS**: pure white base, **SF Pro** system typography, and **cool sky-blue** ambient light instead of warm cream. Depth comes from **glassmorphism** (backdrop blur + white frost borders + blue glow shadows), not flat borders alone.

The hero is a **product demo theater**: a Mac desktop with wallpaper, menubar, and a **Dynamic Island–style lens** that expands into app widgets (Gmail, Calendar). Copy is short, confident, centered; motion sells “point anywhere” and “voice-to-action.”

**Key characteristics**
- White page foundation (`#ffffff`) with soft blue gradient washes (`#f6f9ff` → `#f0f5ff` at ~20–24% opacity)
- System stack: `-apple-system`, `SF Pro Display`, `SF Pro Text`, `system-ui`, sans-serif
- Primary text: slate/gray-900 (`rgb(17, 24, 39)` / `#111827`); deeper headings use `#030712`
- **Liquid glass** controls: conic blue rim gradients, inset highlights, blurred halos
- **Superellipse** corners on lens/card chrome (`corner-shape: superellipse(1.2630)` where supported)
- Demo UI uses **dark acrylic** (`#03050a` menubar, layered tint/shade/sheen) over vivid blue sky
- Motion: springy easings (`cubic-bezier(0.16, 1, 0.3, 1)`), looping cursor/pointer demos, marquee app chips

## 2. Color Palette & Roles

### Primary & neutrals
| Token | Value | Role |
|-------|--------|------|
| **Page white** | `#ffffff` | Body, nav, default sections |
| **Ink primary** | `#111827` (`rgb(17, 24, 39)`) | Headlines, nav, button label |
| **Ink deep** | `#030712` (`rgb(3, 7, 18)`) | Card titles, emphasis |
| **Muted body** | `#5b5b5b` (`--lg-grey`) | Secondary copy, captions |
| **Border hairline** | `rgb(229, 231, 235)` | Default Tailwind gray-200 dividers |

### Sky & glass accents
| Token | Value | Role |
|-------|--------|------|
| **Glass frost** | `rgba(255, 255, 255, 0.72)` | Frosted panel stroke |
| **Glass fill light** | `linear-gradient(rgba(255,255,255,0.984), rgba(250,252,255,0.97))` | Light demo cards |
| **Glass fill soft** | `linear-gradient(rgba(255,255,255,0.86), rgba(250,252,255,0.72))` | Larger panels |
| **Blue wash** | `#2563eb29` | Animated highlight under pointed words |
| **Conic rim dark** | `#143c7880` / `#96c8ff1a` | Glass button border sweep |
| **Blue glow shadow** | `linear-gradient(rgba(0,50,100,0.15), rgba(0,80,150,0.08))` | Pill CTA halo |
| **Sky pill (hero)** | `linear-gradient(#090e1c75, #04071099)` | Floating “sent” toast on wallpaper |

### Dark widget chrome (in-product mocks)
| Token | Value | Role |
|-------|--------|------|
| **Menubar** | `#03050a` | Mac strip |
| **Lens tint base** | `#03050a` → transparent vertical gradient | Island expand |
| **Card shell** | `#202124` | Gmail/Calendar mock |
| **Header bar** | `#404040` / `#292A2D` | App chrome |
| **Google blue** | `#1a73e8`, `#4285F4` | Send / Calendar accent |
| **Chip** | `rgba(138,180,248,0.15)` bg, `#C6DAFC` text | Email chips |

### Liquid-glass CSS variables (site)
```css
--lg-bg-color: #ffffff40;
--lg-highlight: #ffffffbf;
--lg-text: #fff;
--lg-hover-glow: #fff6;
--lg-red: #fb4268;
--lg-grey: #5b5b5b;
```

## 3. Typography Rules

### Font family
- **Marketing UI**: `-apple-system`, `"SF Pro Display"`, `"SF Pro Text"`, `system-ui`, sans-serif
- **In-widget mocks**: `"Google Sans"`, Roboto, Arial (Gmail/Calendar fidelity only)

### Hierarchy (computed from live site)

| Role | Size | Weight | Line height | Letter spacing |
|------|------|--------|-------------|----------------|
| Hero H1 | 76px | 600 | 95px (~1.25) | **-2.28px** |
| Section H2 | 48px | 400 | 48px (1.0) | **-1.2px** |
| Card / quote H3 | 18px | 600 | 28px | normal |
| Nav / body | 16px | 400 | 24px | normal |
| Button label | 18px (1.125rem) | 500 | — | **-0.18px** |
| Widget microcopy | 9.5–11px | 500 | 16px | 0.01em |

### Principles
- **Tight display tracking** on large type; body stays normal
- **Weight contrast**: hero semibold (600), section titles often **regular (400)** at large size for elegance
- **Button text**: medium (500), slight negative tracking, optional subtle `text-shadow: rgba(0,0,0,0.05) 0 0.05em 0.05em`
- Strikethrough / animated phrases in H2 (“Speech-to-text” → “Voice-to-action”) for storytelling

## 4. Glassmorphism & Depth

### Frosted panel (light)
```css
backdrop-filter: blur(16px) saturate(1.4); /* compact cards; use blur(10px) saturate(1.35) for large */
border: 1px solid rgba(255, 255, 255, 0.72);
border-radius: 26px; /* or 32px on hero-scale panels */
box-shadow:
  0 10px 30px rgba(15, 23, 42, 0.10),
  0 1px 4px rgba(15, 23, 42, 0.04),
  inset 0 1px 0 rgba(255, 255, 255, 0.82);
background: linear-gradient(rgba(255,255,255,0.984), rgba(250,252,255,0.97));
```

### Top specular hairline
```css
/* 1px gradient line along top inner edge */
background: linear-gradient(90deg, transparent, rgba(255,255,255,0.95), transparent);
```

### Dark lens / island glass
```css
backdrop-filter: blur(5px) saturate(180%);
border-radius: 0 0 19px 19px; /* closed; opens to 0 0 35px 35px */
/* Layer stack: .lns-glass (blur) + .lns-tint (dark gradient) + .lns-shade + .lns-sheen (white lift) */
```

### Glass filter layer
```css
.glass-filter {
  backdrop-filter: blur(3px) saturate(1.8);
  filter: url("#lg-dist"); /* SVG displacement for liquid distortion */
  animation: glass-filter-settle 0.22s ease-out;
}
```

### Elevation model

| Level | Treatment | Use |
|-------|-----------|-----|
| 0 | Flat white | Page |
| 1 | Hairline + soft gradient overlay | Nav notch sheen |
| 2 | Frosted card (blur 10–16px) | Testimonials, feature cards |
| 3 | Glass pill + blue halo shadow | Primary CTAs |
| 4 | Dark acrylic stack | Hero Mac widget |

## 5. Roundness Scale

| Token | Radius | Use |
|-------|--------|-----|
| **Pill** | `9999px` / `999vw` | Download buttons, tags, sky toasts |
| **Container** | `2rem` (32px) | `.glass-container` |
| **Panel** | `26px`–`32px` | Frosted marketing cards |
| **Hero screen top** | `18px` (15px mobile) | Mac window mock |
| **Widget card** | `24px`–`25px` | Gmail/Calendar inner (`superellipse`) |
| **Lens bottom** | `19px` closed → `35px` open | Island expansion |
| **Small control** | `9999px` | 32px mute / icon circles |
| **Chips** | `10px`–`12px` | Email chips, send button |

## 6. Component Stylings

### Glassmorphic button (primary CTA pattern)

**Structure**: `.glassmorphic-button-wrap` → button + `.glassmorphic-button-shadow` (blurred blue halo).

**Default**
- Shape: full pill (`border-radius: 999vw`)
- Background: `linear-gradient(-75deg, rgba(230,245,255,0.08), rgba(240,250,255,0.2), rgba(230,245,255,0.08))`
- `backdrop-filter: blur(2px)`
- Inset shadow stack:
  - `inset 0 2px 2px rgba(0,0,0,0.05)`
  - `inset 0 -2px 2px rgba(255,255,255,0.5)`
  - `0 4px 2px -2px rgba(0,0,0,0.2)`
  - `inset 0 0 1.6px 4px rgba(210,235,255,0.25)`
- Label: 18px, weight 500, `#323232`, padding inline `1.5em`–`1.75em`
- Pseudo border: conic gradient `#143c7880` / `#96c8ff1a` + linear `#dcf0ff80` → `#f0faff80`

**Primary variant** (`.is-primary`)
- Fill: `linear-gradient(-75deg, #23252e, #14161d 55%, #23252e)`
- Label: `#ffffff`, no text-shadow

**Hover**
- `transform: scale(0.975)`
- Stronger inset + outer shadow; conic angle animates (`--angle-1: -125deg`)
- Halo blur tightens (`blur(clamp(2px, 0.0625em, 6px))`)
- Transition: `0.4s cubic-bezier(0.25, 1, 0.5, 1)`

**Nav Download** uses the same glass pill at ~230–236px width on desktop.

### Navigation
- White bar, logo wordmark left, text links 16px/400
- “Product” dropdown chevron
- Y Combinator badge: small muted label + orange mark
- Right CTA: compact glass pill (dark `.is-primary` or light glass)

### Hero product widget (Mac demo)

**Screen container** (`.hero-screen`)
- Width: `max(340px, min(74vw, 1120px))`, height ~283px
- Top radius 18px; wallpaper fades via long **mask-image** gradient (85%→100% alpha steps)
- Menubar 18px `#03050a`; optional 2px gradient “bezel” line `#e8e8e6` → `#a8a8a4`

**Lens / notch** (`.lns`)
- Closed: width `min(38%, 330px)`, height 44px
- Open: width `min(46%, 430px)`, height 233px
- Transitions: 300ms default; **460ms** open with `cubic-bezier(0.16, 1, 0.3, 1)`
- Inner card scales from 0.97 → 1 when open; shade fades, sheen appears

**Floating sky toast** (`.hero-screen-sky-sent-inner`)
- Pill, white text, weight ~590
- `backdrop-filter: blur(14px) saturate(180%)`
- Inset highlight + soft drop shadow `#03103057`
- Enter: `0.32s cubic-bezier(0.22, 0.9, 0.3, 1)`; exit upward fade

**Cursor hint**
- SVG pointer: fill `#0f172a`, white stroke 2.4px, drop-shadow `#04163e73`
- Wander loop 7.6s + jiggle rotation keyframes; hidden under `prefers-reduced-motion`

### Feature cards & quotes
- Light glass panels with avatar, **decorative glass highlight** on photos
- Horizontal **marquee** of app icons + labels (Gmail, Slack, Linear, …) — infinite scroll
- Testimonial grid: frosted cards, 18px semibold names

### Privacy section
- iOS-style **toggle switches** on glass cards
- Group labels: “On your device” vs “Optional sharing”

### FAQ
- Accordion rows, collapsed by default
- Large section H2 “Questions?”

### Footer CTA
- Full-bleed **blue ocean/sky** photography
- Repeated glass “Download for Mac” pill
- **`fn` key** visual as glass chip in headline
- “Ask ChatGPT / Claude / Perplexity” text links

## 7. Motion & Animation

| Name | Duration / easing | Effect |
|------|-------------------|--------|
| `glass-filter-settle` | 0.22s ease-out | Glass layer fade-in |
| `hero-cursor-hint-in` | 0.6s `cubic-bezier(0.22,1,0.36,1)`, delay 1.6s | Cursor appear |
| `hero-cursor-hint-wander` | 7.6s infinite | Cursor path across hero |
| `hero-cursor-hint-jiggle` | 7.6s infinite | Subtle rotation |
| `hero-sky-sent-in/out` | 0.32s / 0.4s | Toast enter/leave |
| `ptr-wash-cycle` | 5.4s linear infinite | Blue highlight sweep on words |
| `ptr-cycle` | 5.4s | Cursor moves between headline targets |
| LNS open | 460ms `cubic-bezier(0.16, 1, 0.3, 1)` | Island expand |
| LNS close | 300ms `cubic-bezier(0.5, 0, 0.75, 0)` | Island collapse |
| Button hover | 400ms `cubic-bezier(0.25, 1, 0.5, 1)` | Scale + shadow + rim |
| Hero screen exit | 500ms ease-out | Demo fade on scroll |

**Reduced motion**: LNS and cursor animations collapse to ~1ms transitions; cursor hint hidden on coarse pointer / reduced motion.

## 8. Layout Principles

- **Centered marketing column**, generous vertical rhythm between story sections
- Hero: headline → glass CTA → full-width demo (demo is the visual anchor)
- Sections alternate **text-led** blocks and **full-bleed media** (sky, ocean footer)
- App marquee: full-width horizontal band
- Spacing follows Tailwind scale (4px base); hero uses large top padding for nav clearance

## 9. Do's and Don'ts

### Do
- Use **system SF stack** and tight negative tracking on display sizes
- Build CTAs as **glass pills** with blue halo + conic rim, not flat solid rectangles
- Pair **blur + saturate(1.35–1.8)** with white semi-transparent borders
- Use **layered gradients** (tint/shade/sheen) for dark floating widgets
- Animate **pointer/context** stories with slow loops and spring easings
- Fade product imagery into white/sky with **mask gradients**, not hard crops

### Don't
- Don't revert to warm cream/Lovable palette for VoiceOS-style pages
- Don't use heavy opaque drop shadows without frosted border — reads as pre-2020 SaaS
- Don't square off primary buttons — pills are the brand shape
- Don't skip `prefers-reduced-motion` fallbacks on cursor/island animations
- Don't mix widget mock fonts into marketing chrome (keep Google Sans inside demos only)

## 10. Agent Prompt Guide

### Quick tokens
- Background: `#ffffff` + optional `linear-gradient(rgba(246,249,255,0.24), rgba(240,245,255,0.2))`
- Text: `#111827` headings, `#5b5b5b` secondary
- Glass border: `1px solid rgba(255,255,255,0.72)`
- Blur: `blur(16px) saturate(1.4)` (cards), `blur(2px)` (buttons)
- Radius: pills `9999px`, panels `26–32px`, widget cards `24px`
- CTA shadow halo: blue `rgba(0,50,100,0.15)` → `rgba(0,80,150,0.08)`

### Example prompts
- “Hero: 76px/600, -2.28px tracking, centered. Below, glass pill CTA with conic blue border and soft blue glow shadow. Mac mock with 18px top radius, menubar `#03050a`, wallpaper mask fade.”
- “Frosted testimonial card: 26px radius, blur 16px saturate 1.4, white 72% border, inset top highlight. Name 18px/600.”
- “Dark island widget: 44px collapsed height, expands to 233px with 460ms spring; backdrop blur 5px; bottom corners superellipse 19px→35px.”

### Iteration checklist
1. White + cool blue ambient, not warm neutrals
2. Pills for all primary actions
3. Display tracking −2px at 76px, −1.2px at 48px
4. Three-part glass: **blur layer + gradient fill + specular inset line**
5. Motion on hero only — subtle elsewhere
6. Respect reduced motion
