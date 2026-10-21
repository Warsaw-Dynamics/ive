<!-- Sample comment -->
<template>
  <section class="cart">
    <h1>{{ title.toUpperCase() }} &amp; more</h1>
    <CartItem
      v-for="(item, index) in items"
      :key="item.id"
      v-bind:[attribute]="item.price"
      @click.stop="remove(index)"
    />
    <input v-model.trim="coupon" disabled>
    <base-button #default="{ label }">{{ label }}</base-button>
  </section>
</template>

<script setup lang="ts">
import { ref } from 'vue';

interface Item { id: number; price: number }

const title = 'IVE';
const attribute = 'data-price';
const coupon = ref('');
const items = ref<Item[]>([{ id: 1, price: 42 }]);

function remove(index: number): void {
  items.value.splice(index, 1);
}
</script>

<style scoped lang="scss">
$accent: #3c3f41;
.cart h1 { color: $accent; }
</style>
