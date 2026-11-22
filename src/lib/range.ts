// Svelte action for M3 sliders: paints the active part of the track.
// Usage: <input type="range" use:fill={value} ...>
export function fill(node: HTMLInputElement, _value?: unknown) {
  const paint = () => {
    const min = +node.min || 0;
    const max = node.max === "" ? 100 : +node.max;
    node.style.setProperty("--p", `${((+node.value - min) / (max - min || 1)) * 100}%`);
  };
  paint();
  node.addEventListener("input", paint);
  return {
    update: () => queueMicrotask(paint),
    destroy: () => node.removeEventListener("input", paint),
  };
}
