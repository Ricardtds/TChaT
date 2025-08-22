<script>
    import { createEventDispatcher } from "svelte";

    export let items = ["Início", "Produtos", "Sobre Nós", "Contato"];

    let activeItem = items[0];

    const dispatch = createEventDispatcher();

    function selectItem(item) {
        activeItem = item;
        dispatch("tabChange", item);
    }
</script>

<header>
    <nav>
        <ul>
            {#each items as item}
                <li>
                    <button
                        on:click={() => selectItem(item)}
                        class:active={activeItem === item}
                    >
                        {item}
                    </button>
                </li>
            {/each}
        </ul>
    </nav>
</header>

<style>
    header {
        background-color: #f8f9fa;
        border-bottom: 1px solid #dee2e6;
        padding: 0 1rem;
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
            Oxygen, Ubuntu, Cantarell, "Open Sans", "Helvetica Neue", sans-serif;
    }

    nav ul {
        display: flex;
        list-style: none;
        margin: 0;
        padding: 0;
    }

    li {
        margin-right: 0.5rem;
    }

    button {
        background: none;
        border: none;
        padding: 1rem 1.25rem;
        cursor: pointer;
        font-size: 1rem;
        color: #495057;
        position: relative;
        transition: color 0.2s ease-in-out;
    }

    button:hover {
        color: #007bff;
    }

    /*
   * Estilo da aba ativa.
  */
    button.active {
        color: #0056b3;
        font-weight: 600;
    }

    button.active::after {
        content: "";
        position: absolute;
        bottom: -1px; /* Alinha com a borda do header */
        left: 0;
        right: 0;
        height: 3px;
        background-color: #007bff;
        border-radius: 2px;
    }
</style>
