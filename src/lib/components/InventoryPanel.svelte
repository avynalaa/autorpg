<script lang="ts">
  import { inventory } from '../stores/gameStore';
  import { equipItem, removeInventoryItem } from '../stores/gameStore';
  import type { InventorySlot, Item } from '../types';

  let selectedSlot = $state<number | null>(null);

  const categoryIcon: Record<string, string> = {
    Weapon: '\u2694', Armor: '\uD83D\uDEE1', Consumable: '\uD83E\uDDEA', Quest: '\uD83D\uDCDC', Tool: '\uD83D\uDD27', Key: '\uD83D\uDD11', Misc: '\u25CB'
  };

  function selectSlot(idx: number) {
    selectedSlot = selectedSlot === idx ? null : idx;
  }

  async function handleEquip(idx: number) {
    await equipItem(idx);
    selectedSlot = null;
  }

  async function handleDrop(idx: number) {
    await removeInventoryItem(idx);
    selectedSlot = null;
  }

  function itemBgColor(item: Item): string {
    switch (item.category) {
      case 'Weapon': return '#3b1f1f';
      case 'Armor': return '#1f2a3b';
      case 'Consumable': return '#1f3b1f';
      case 'Quest': return '#2a1f3b';
      default: return '#1e1e2e';
    }
  }
</script>

{#if $inventory}
  <div class="panel inv-panel">
    <div class="panel-title">Inventory
      <span class="weight">{$inventory.slots.reduce((a: number, s: InventorySlot) => a + (s.item ? s.item.weight * s.item.quantity : 0), 0).toFixed(1)}kg / {$inventory.max_weight}kg</span>
    </div>

    <!-- Equipped items -->
    {#if Object.keys($inventory.equipped).length > 0}
      <div class="equipped-section">
        <div class="sub-title">Equipped</div>
        <div class="equipped-grid">
          {#each Object.entries($inventory.equipped) as [slot, item]}
            <div class="equipped-item">
              <div class="eq-slot-label">{slot}</div>
              <div class="eq-item-name">{item.name}</div>
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Backpack grid -->
    <div class="inv-grid">
      {#each $inventory.slots as slot}
        <button
          class="inv-slot"
          class:occupied={slot.item !== null}
          class:selected={selectedSlot === slot.index}
          style={slot.item ? `background:${itemBgColor(slot.item)}` : ''}
          onclick={() => slot.item && selectSlot(slot.index)}
          title={slot.item?.name ?? ''}
        >
          {#if slot.item}
            <span class="item-icon">{categoryIcon[slot.item.category] ?? '\u25CB'}</span>
            <span class="item-name">{slot.item.name.slice(0, 10)}</span>
            {#if slot.item.quantity > 1}
              <span class="item-qty">\u00D7{slot.item.quantity}</span>
            {/if}
          {/if}
        </button>
      {/each}
    </div>

    <!-- Item detail popup -->
    {#if selectedSlot !== null}
      {@const slot = $inventory.slots[selectedSlot]}
      {#if slot?.item}
        <div class="item-detail">
          <div class="detail-name">{slot.item.name}</div>
          <div class="detail-cat">{slot.item.category} \u00B7 {slot.item.weight}kg \u00B7 {slot.item.value}g</div>
          {#if slot.item.description}
            <div class="detail-desc">{slot.item.description}</div>
          {/if}
          {#if Object.keys(slot.item.stat_mods).length > 0}
            <div class="detail-mods">
              {#each Object.entries(slot.item.stat_mods) as [k,v]}
                <span class="mod-badge" class:positive={v > 0} class:negative={v < 0}>
                  {k}: {v > 0 ? '+' : ''}{v}
                </span>
              {/each}
            </div>
          {/if}
          <div class="detail-actions">
            {#if slot.item.equip_slot}
              <button class="btn btn-blue" onclick={() => handleEquip(selectedSlot!)}>Equip</button>
            {/if}
            <button class="btn btn-red" onclick={() => handleDrop(selectedSlot!)}>Drop</button>
            <button class="btn btn-grey" onclick={() => selectedSlot = null}>&#x2715;</button>
          </div>
        </div>
      {/if}
    {/if}
  </div>
{/if}

<style>
  .panel { background: var(--bg-surface); border: 1px solid var(--border); border-radius: var(--radius-md); padding: 12px; box-shadow: var(--shadow-panel); font-family: var(--font-ui); }
  .panel-title { font-size: var(--font-size-xs); text-transform: uppercase; letter-spacing: 1px; color: var(--text-muted); margin-bottom: 8px; display: flex; justify-content: space-between; }
  .weight { color: var(--text-secondary); font-size: var(--font-size-xs); }
  .sub-title { font-size: var(--font-size-xs); color: var(--text-muted); margin-bottom: 4px; }
  .equipped-section { margin-bottom: 10px; }
  .equipped-grid { display: flex; flex-wrap: wrap; gap: 4px; }
  .equipped-item { background: var(--bg-elevated); border: 1px solid var(--border-accent); border-radius: var(--radius-xs); padding: 4px 8px; font-size: var(--font-size-sm); }
  .eq-slot-label { color: var(--text-muted); font-size: 9px; }
  .eq-item-name { color: var(--accent); }
  .inv-grid { display: grid; grid-template-columns: repeat(6, 1fr); gap: 3px; }
  .inv-slot {
    aspect-ratio: 1; background: var(--bg-elevated); border: 1px solid var(--border); border-radius: var(--radius-xs);
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    cursor: default; padding: 2px; font-size: 9px; color: var(--text-secondary); position: relative;
    transition: border-color var(--transition);
  }
  .inv-slot.occupied { cursor: pointer; }
  .inv-slot.occupied:hover { border-color: var(--accent); }
  .inv-slot.selected { border-color: var(--special); box-shadow: 0 0 0 1px var(--special); }
  .item-icon { font-size: 14px; line-height: 1; }
  .item-name { font-size: 8px; text-align: center; color: var(--text-primary); overflow: hidden; max-width: 100%; }
  .item-qty { position: absolute; bottom: 1px; right: 3px; font-size: 8px; color: var(--warning); }
  .item-detail { margin-top: 8px; background: var(--bg-elevated); border: 1px solid var(--bg-active); border-radius: var(--radius-sm); padding: 10px; }
  .detail-name { font-size: var(--font-size-md); font-weight: 600; color: var(--text-primary); }
  .detail-cat { font-size: var(--font-size-xs); color: var(--text-muted); margin-bottom: 4px; }
  .detail-desc { font-size: var(--font-size-sm); color: var(--text-secondary); margin-bottom: 6px; }
  .detail-mods { display: flex; gap: 4px; flex-wrap: wrap; margin-bottom: 6px; }
  .mod-badge { font-size: var(--font-size-xs); padding: 1px 6px; border-radius: var(--radius-full); background: var(--bg-hover); }
  .mod-badge.positive { color: var(--success); }
  .mod-badge.negative { color: var(--danger); }
  .detail-actions { display: flex; gap: 6px; }
  .btn { padding: 4px 10px; border: none; border-radius: var(--radius-xs); font-size: var(--font-size-sm); cursor: pointer; }
  .btn-blue { background: var(--accent-dim); color: var(--accent); }
  .btn-red { background: var(--item-weapon-bg); color: var(--danger); }
  .btn-grey { background: var(--bg-hover); color: var(--text-secondary); }
</style>
