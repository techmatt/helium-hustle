// The Buildings panel: the Land bar, then a collapsible section per category of cards.

import type { BuildingView, View } from '../engine';
import { capitalize, num } from '../format';
import { act, actAll, seen, store, type Store } from '../store';
import { AXIS_COLORS, CostRows, FlowLines, Section, Square, freshClass, name } from './common';

export function Buildings({ s }: { s: Store }) {
  const v = s.view;
  const land = v.resources.find((r) => r.id === 'land');
  const shown = v.buildings.filter((b) => b.visible || b.owned > 0);
  // Categories in content order of their first building.
  const categories = [...new Set(v.buildings.map((b) => b.category))];
  const tags = v.nav.some((t) => t.id === 'ideologies' && t.visible);
  return (
    <div class="panel" data-panel="buildings">
      <h2>Buildings</h2>
      <div class="land-bar">
        <Square id="land" size={16} />
        <strong>Land</strong>
        <span class="muted">{num(land?.amount ?? '0')} free</span>
        <span class="land-offer">
          +{num(v.land_per_purchase)} land for {num(v.land_price)} credits
        </span>
        <button type="button" data-testid="buy-land" disabled={!v.can_buy_land} onClick={() => act({ type: 'buy_land' })}>
          Buy Land
        </button>
      </div>
      {categories.map((cat) => {
        const cards = shown.filter((b) => b.category === cat);
        if (cards.length === 0) return null;
        return (
          <Section key={cat} title={cat.toUpperCase()} dark>
            <div class="cards">
              {cards.map((b) => (
                <Card key={b.id} v={v} b={b} tags={tags} fresh={s.fresh} />
              ))}
            </div>
          </Section>
        );
      })}
    </div>
  );
}

function Card({ v, b, tags, fresh }: { v: View; b: BuildingView; tags: boolean; fresh: Set<string> }) {
  const key = `building:${b.id}`;
  const atLimit = b.max_count !== null && b.owned >= b.max_count;
  // From the engine's count now, not this render's: quick clicks must each count.
  const setActive = (by: number) => {
    const now = store.view.buildings.find((x) => x.id === b.id);
    if (now) act({ type: 'set_active', building: b.id, active: Math.max(0, Math.min(now.owned, now.active + by)) });
  };
  return (
    <div
      class={`card building ${b.affordable ? 'can-buy' : 'cannot-buy'}${freshClass(fresh, key)}`}
      data-id={b.id}
      data-owned={b.owned}
      data-short={b.short.join(' ')}
      onMouseEnter={() => seen(key)}
    >
      <div class="card-head">
        <h3>{b.name}</h3>
        <span class="owned" title="active / owned">
          ({b.active}/{b.owned})
        </span>
        <button type="button" class="tiny" title="Run one fewer" disabled={b.active === 0} onClick={() => setActive(-1)}>
          −
        </button>
        <button type="button" class="tiny" title="Run one more" disabled={b.active >= b.owned} onClick={() => setActive(1)}>
          +
        </button>
      </div>
      <p class="desc">{b.description}</p>
      <div class="effects">
        <FlowLines v={v} amounts={b.produces} sign={1} />
        <FlowLines v={v} amounts={b.upkeep} sign={-1} />
        <FlowLines v={v} amounts={b.stores} sign={1} suffix=" storage" />
        <FlowLines v={v} amounts={b.grants} sign={1} suffix="" />
      </div>
      {tags && b.ideology && (
        <span class="tag" style={{ background: AXIS_COLORS[b.ideology] }}>
          {capitalize(b.ideology)}
        </span>
      )}
      {b.stall && (
        <span class={`stall stall-${b.stall}`}>{b.stall === 'input_starved' ? 'Short of inputs' : 'Output at cap'}</span>
      )}
      <hr />
      {atLimit ? (
        <p class="muted">At its limit ({b.max_count}).</p>
      ) : (
        <>
          <CostRows v={v} cost={b.cost} short={b.short} />
          <table class="costs">
            <tbody>
              <tr class={b.short.includes('land') ? 'short' : ''}>
                <td>
                  <Square id="land" /> {name(v, 'land')}
                </td>
                <td class="num">{b.land}</td>
              </tr>
            </tbody>
          </table>
        </>
      )}
      <div class="card-actions">
        <button type="button" class="buy" data-testid={`buy-${b.id}`} disabled={!b.affordable} onClick={() => act(b.buy)}>
          {b.unlocked ? 'Buy' : 'Locked'}
        </button>
        {b.owned > 0 && (
          <>
            <button type="button" class="tiny sell" title="Sell one (refunds its land only)" onClick={() => act({ type: 'sell', building: b.id })}>
              Sell 1
            </button>
            <button
              type="button"
              class="tiny sell"
              title="Sell every one (refunds their land only)"
              onClick={() => {
                if (confirm(`Sell all ${b.owned} ${b.name}? Only their land comes back.`))
                  actAll(Array.from({ length: b.owned }, () => ({ type: 'sell', building: b.id })));
              }}
            >
              Sell All
            </button>
          </>
        )}
      </div>
    </div>
  );
}
