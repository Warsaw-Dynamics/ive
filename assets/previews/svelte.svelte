<!-- Sample comment -->
<script lang="ts">
  import Counter from './Counter.svelte';

  interface Item { id: number; name: string }

  let items: Item[] = [{ id: 1, name: 'IVE' }];
  let filter = $state('');
  const loading: Promise<number> = fetch('/api').then((r) => r.status);
</script>

<svelte:window on:keydown={(event) => console.log(event.key)} />

<section class="list" class:empty={items.length === 0}>
  <input bind:value={filter} placeholder="Filter &amp; sort" disabled />
  {#each items as item (item.id)}
    <Counter {...item} label="Item {item.name}" />
  {:else}
    <p>Nothing yet &amp; no filter</p>
  {/each}
  {#if filter.length > 2}
    {@html filter.toUpperCase()}
  {:else if filter}
    {@const size = filter.length}
    <span>{size}</span>
  {/if}
  {#await loading then status}
    <b>{status}</b>
  {:catch error}
    {@debug error}
  {/await}
  {#snippet row(entry: Item)}<em>{entry.name}</em>{/snippet}
  {#key filter}{@render row(items[0])}{/key}
</section>

<style lang="scss">
  $gap: 4px;
  .list { padding: $gap; }
</style>
