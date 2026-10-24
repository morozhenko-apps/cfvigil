// Pure keyboard navigation: avoid coupling focus movement to DOM updates.
// All visible resource actions remain reachable by Tab; arrow keys are optional.
export function nextResourceIndex(current, count, key) {
  if (!Number.isInteger(count) || count <= 0) return -1;
  switch (key) {
    case "Home": return 0;
    case "End": return count - 1;
    case "ArrowDown": return current < 0 ? 0 : (current + 1) % count;
    case "ArrowUp": return current < 0 ? count - 1 : (current + count - 1) % count;
    default: return -1;
  }
}
