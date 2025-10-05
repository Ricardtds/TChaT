<script lang="ts">
    import type { TabItem } from "$lib/models/TabItem";

    let {
        stream,
        removeStreammer,
    }: { stream: TabItem; removeStreammer: (stream: TabItem) => void } =
        $props();
</script>

<div
    class="flex border-[#374151] border-2 w-fit rounded-t-lg p-1 items-center gap-1"
>
    <div class="">{stream.name}</div>
    {#if stream.status}
        <div class="bg-[#ff0000] w-2 h-2 rounded-full"></div>
    {/if}
    <button
        onclick={() => {
            removeStreammer(stream);
        }}>x</button
    >
</div>
