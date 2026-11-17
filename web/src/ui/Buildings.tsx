// The Buildings panel: the Land bar, then a collapsible section per category of cards.

import type { BuildingView, View } from '../engine';
import { capitalize, num } from '../format';
import { act, actAll, seen, store, type Store } from '../store';
import { buildingArt } from './buildingArt';
import { CostRows, FlowLines, ResChip, Section, cardButton, freshClass, name, navShown, shortText, useHint } from './common';
import { Chip } from './icons';

/** `?all-cards` in the URL (the QA checker's): every building's card, revealed or not, so
 * every card picture can be looked at. Display only; off in normal use. */
const ALL_CARDS = new URLSearchParams(location.search).has('all-cards');

export function Buildings({ s }: { s: Store }) {
  const v = s.view;
  const land = v.resources.find((r) => r.id === 'land');
  const shown = v.buildings.filter((b) => b.visible || b.owned > 0 || ALL_CARDS);
  // Categories in content order of their first building.
  const categories = [...new Set(v.buildings.map((b) => b.category))];
  const tags = navShown(v, 'ideologies');
  return (
    <div class="panel" data-panel="buildings">
      <h2>Buildings</h2>
      {/* The whole row buys land, like a green building card (the engine's view at click
          time, so quick clicks each count once). */}
      <div
        class={`land-bar${v.can_buy_land ? ' can-buy' : ''}`}
        data-testid="land-bar"
        aria-disabled={!v.can_buy_land}
        title={v.can_buy_land ? 'Click to buy land' : undefined}
        {...cardButton(() => {
          if (store.view.can_buy_land) act({ type: 'buy_land' });
        })}
      >
        <ResChip v={v} id="land" size={16} />
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

/** Why a grey card buys nothing, in a few words. */
function whyNot(v: View, b: BuildingView): string {
  if (b.at_limit) return 'At its limit.';
  if (!b.unlocked) return 'Locked.';
  if (b.short.length === 0) return 'Not affordable.';
  return `Short of ${b.short.map((r) => name(v, r)).join(', ')}.`;
}

function Card({ v, b, tags, fresh }: { v: View; b: BuildingView; tags: boolean; fresh: Set<string> }) {
  const key = `building:${b.id}`;
  const art = buildingArt(b.id);
  const atLimit = b.at_limit;
  // From the engine's count now, not this render's: quick clicks must each count.
  const setActive = (by: number) => {
    const now = store.view.buildings.find((x) => x.id === b.id);
    if (now) act({ type: 'set_active', building: b.id, active: Math.max(0, Math.min(now.owned, now.active + by)) });
  };
  // The card is the Buy button (`cardButton`): a click buys one, Shift up to ten. Each buy
  // reads the engine's view now, not this render's, so two quick clicks buy two and a
  // double-click buys what its two clicks would, never more.
  const [hint, showHint] = useHint();
  const buy = (times: number) => {
    for (let i = 0; i < times; i++) {
      const now = store.view.buildings.find((x) => x.id === b.id);
      if (!now?.affordable) {
        if (i === 0 && now) showHint(whyNot(store.view, now));
        return;
      }
      if (!act(now.buy)) return;
    }
  };
  return (
    <div
      class={`card building ${b.affordable ? 'can-buy' : 'cannot-buy'}${freshClass(fresh, key)}`}
      data-id={b.id}
      data-owned={b.owned}
      data-short={b.short.join(' ')}
      data-testid={`buy-${b.id}`}
      aria-disabled={!b.affordable}
      title={b.affordable ? 'Click to build one (Shift-click: up to 10)' : undefined}
      onMouseEnter={() => seen(key)}
      {...cardButton(buy, 10)}
    >
      <div class="card-head">
        <h3>{b.name}</h3>
        {tags && b.ideology && (
          <span class="ideology-badge" data-ideology={b.ideology}>
            <Chip id={b.ideology} title={capitalize(b.ideology)} size={18} />
          </span>
        )}
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
      {/* The picture sits under the title, beside the description and flows. */}
      <div class="card-body">
        {art && <img class="card-art" src={art} alt="" width={96} height={96} draggable={false} />}
        <div class="card-text">
          <p class="desc">{b.description}</p>
          <div class="effects">
            <FlowLines v={v} amounts={b.produces} sign={1} />
            <FlowLines v={v} amounts={b.upkeep} sign={-1} />
            <FlowLines v={v} amounts={b.stores} sign={1} suffix=" cap" />
            <FlowLines v={v} amounts={b.grants} sign={1} suffix="" />
          </div>
        </div>
      </div>
      {/* The status row's space is always kept, so a card never jumps as a stall comes and goes. */}
      <div class="card-status" data-card-status title={b.starved ? shortText(v, b.starved) : undefined}>
        {b.starved ? (
          <span class="stall stall-input_starved">
            running at {Math.floor(Number(b.starved.ran) * 100)}% · <ResChip v={v} id={b.starved.resource} /> short
          </span>
        ) : b.stall === 'input_starved' ? (
          <span class="stall stall-input_starved">Short of inputs</span>
        ) : b.stall === 'output_capped' ? (
          <span class="stall stall-output_capped">Output at cap</span>
        ) : null}
      </div>
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
                  <ResChip v={v} id="land" /> {name(v, 'land')}
                </td>
                <td class="num">{b.land}</td>
              </tr>
            </tbody>
          </table>
        </>
      )}
      <div class="card-actions">
        <span class={hint ? 'buy-hint short' : 'buy-hint muted'}>{hint ?? (b.unlocked ? '' : 'Locked')}</span>
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
