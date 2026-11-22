<script lang="ts" module>
  // Material Symbols (Apache-2.0), rounded style. Only the icons listed here are bundled.
  const files = import.meta.glob(
    "../../node_modules/@material-symbols/svg-400/rounded/{tune,graphic_eq,settings_input_component,settings,add,delete,play_arrow,edit,bolt,check,close,sensors,content_copy,layers,link_off,refresh,usb,music_note,videocam,keyboard,warning,drag_indicator,touch_app}.svg",
    { query: "?raw", import: "default", eager: true },
  ) as Record<string, string>;
  const icons: Record<string, string> = {};
  for (const [path, svg] of Object.entries(files)) {
    icons[path.split("/").pop()!.replace(".svg", "")] = svg.replace(/ width="48" height="48"/, "");
  }
</script>

<script lang="ts">
  let { name, size = 24 }: { name: string; size?: number } = $props();
</script>

<span class="icon" style="width:{size}px;height:{size}px" aria-hidden="true">{@html icons[name] ?? ""}</span>

<style>
  .icon { display: inline-flex; flex: none; }
  .icon :global(svg) { width: 100%; height: 100%; fill: currentColor; }
</style>
