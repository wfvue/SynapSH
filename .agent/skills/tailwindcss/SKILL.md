---
name: tailwindcss
description: TailwindCSS v4 utility-first CSS framework with Vite plugin integration. Use when styling components, configuring Tailwind, or working with utility classes.
---

# TailwindCSS Skill

## Overview
TailwindCSS v4 is a utility-first CSS framework integrated via the `@tailwindcss/vite` plugin.

## Installation (Already Done)
```bash
pnpm add tailwindcss @tailwindcss/vite
```

## Configuration

### Vite Config (`vite.config.ts`)
```typescript
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [vue(), tailwindcss()],
});
```

### CSS Entry (`src/style.css`)
```css
@import "tailwindcss";
```

### Import in `main.ts`
```typescript
import "./style.css";
```

## Usage Examples

### Basic Utilities
```html
<div class="flex items-center justify-between p-4 bg-gray-100 rounded-lg">
  <h1 class="text-2xl font-bold text-gray-900">Title</h1>
  <button class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600">
    Click
  </button>
</div>
```

### Responsive Design
```html
<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
  <!-- Cards -->
</div>
```

### Dark Mode
```html
<div class="bg-white dark:bg-gray-800 text-gray-900 dark:text-white">
  Content
</div>
```

### Custom Theme (in `style.css`)
```css
@import "tailwindcss";

@theme {
  --color-primary: #3b82f6;
  --color-secondary: #10b981;
  --font-sans: 'Inter', sans-serif;
}
```

## Key Utility Categories
- **Layout**: `flex`, `grid`, `container`, `hidden`, `block`
- **Spacing**: `p-*`, `m-*`, `gap-*`, `space-x-*`
- **Sizing**: `w-*`, `h-*`, `min-w-*`, `max-h-*`
- **Typography**: `text-*`, `font-*`, `leading-*`, `tracking-*`
- **Colors**: `bg-*`, `text-*`, `border-*`
- **Borders**: `border-*`, `rounded-*`, `ring-*`
- **Effects**: `shadow-*`, `opacity-*`, `blur-*`
- **Transitions**: `transition-*`, `duration-*`, `ease-*`
- **Transforms**: `scale-*`, `rotate-*`, `translate-*`

## Responsive Breakpoints
- `sm:` - 640px+
- `md:` - 768px+
- `lg:` - 1024px+
- `xl:` - 1280px+
- `2xl:` - 1536px+

## State Variants
- `hover:` - Hover state
- `focus:` - Focus state
- `active:` - Active state
- `disabled:` - Disabled state
- `first:` / `last:` - First/last child
- `odd:` / `even:` - Odd/even children

## Resources
- [Official Docs](https://tailwindcss.com/docs)
- [Playground](https://play.tailwindcss.com/)
