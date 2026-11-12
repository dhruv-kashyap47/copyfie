// Lucide icon paths (ISC), copied from lucide-static v1.72.0.
// Upstream: https://lucide.dev — see THIRD_PARTY.md for the licence.
//
// Only the paths Copyfie uses are kept; there is no icon package dependency.
// Each value is the inner markup for a 24×24, stroke-based SVG.

export const icons = {
  search: '<path d="m21 21-4.34-4.34" /><circle cx="11" cy="11" r="8" />',
  x: '<path d="M18 6 6 18" /><path d="m6 6 12 12" />',
  text: '<path d="M21 5H3" /><path d="M15 12H3" /><path d="M17 19H3" />',
  image:
    '<rect width="18" height="18" x="3" y="3" rx="2" ry="2" /><circle cx="9" cy="9" r="2" /><path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21" />',
  files:
    '<path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z" />',
  pin: '<path d="M12 17v5" /><path d="M9 10.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V7a1 1 0 0 1 1-1 2 2 0 0 0 0-4H8a2 2 0 0 0 0 4 1 1 0 0 1 1 1z" />',
} as const;

export type IconName = keyof typeof icons;
