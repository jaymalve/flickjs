import { UseStateSide } from './use-state-side';
import { FxSide } from './fx-side';

export function App() {
  return (
    <div style={{ maxWidth: 960, margin: '0 auto', padding: '48px 20px' }}>
      <header style={{ textAlign: 'center', marginBottom: 48 }}>
        <h1
          style={{
            fontSize: 32,
            fontWeight: 700,
            letterSpacing: '-0.04em',
            color: '#fafafa'
          }}
        >
          Re-render Showdown
        </h1>
        <p
          style={{
            fontSize: 14,
            color: '#a6a6b8',
            marginTop: 8,
            maxWidth: 480,
            margin: '8px auto 0',
            lineHeight: 1.5
          }}
        >
          Type in both inputs, then click "Stream AI" on both sides. Watch the render counts
          diverge.
        </p>
      </header>

      <div
        style={{
          display: 'grid',
          gridTemplateColumns: '1fr 1fr',
          gap: 24
        }}
      >
        <UseStateSide />
        <FxSide />
      </div>
    </div>
  );
}

const mono: React.CSSProperties = {
  fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace',
  fontSize: 12,
  background: 'hsl(240 4% 10%)',
  padding: '2px 6px',
  borderRadius: 3
};
