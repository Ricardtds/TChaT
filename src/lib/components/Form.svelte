<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";

    invoke;
</script>

<form action="" method="post" class="w-full h-fit bg-[#585858]">
    <textarea
        name="message"
        id="message"
        placeholder="Escreva sua mensagem..."
        class="w-full p-2 rounded resize-none max-h-32 text-white placeholder:italic placeholder:text-white/50 placeholder:text-shadow-lg"
    ></textarea>
</form>
