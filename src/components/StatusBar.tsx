import { useGameStore } from '../stores/gameStore';
import type { Country } from '../types';

function formatPopulation(num: number): string {
  if (num >= 1e9) return `${(num / 1e9).toFixed(2)}B`;
  if (num >= 1e6) return `${(num / 1e6).toFixed(1)}M`;
  if (num >= 1e3) return `${(num / 1e3).toFixed(1)}K`;
  return num.toString();
}

function formatGdp(num: number): string {
  if (num >= 1e12) return `$${(num / 1e12).toFixed(2)}T`;
  if (num >= 1e9) return `$${(num / 1e9).toFixed(1)}B`;
  if (num >= 1e6) return `$${(num / 1e6).toFixed(1)}M`;
  return `$${num}`;
}

function MilitaryBar({
  label,
  value,
  max = 1000,
  color,
}: {
  label: string;
  value: number;
  max?: number;
  color: string;
}) {
  const pct = Math.min((value / max) * 100, 100);
  return (
    <div className="statbar-item">
      <span className="statbar-label">{label}</span>
      <div className="statbar-track">
        <div
          className="statbar-fill"
          style={{ width: `${pct}%`, backgroundColor: color }}
        />
      </div>
      <span className="statbar-value">{value.toLocaleString()}</span>
    </div>
  );
}

const StatusBar: React.FC = () => {
  const playerCountry = useGameStore((s) => s.playerCountry);

  if (!playerCountry) return null;

  const c: Country = playerCountry;

  return (
    <div className="statusbar">
      <div className="statusbar-item">
        <span className="statusbar-flag">{c.id}</span>
        <span className="statusbar-name">{c.name}</span>
      </div>
      <div className="statusbar-separator" />
      <div className="statusbar-item">
        <span className="statusbar-label-text">Pop</span>
        <span className="statusbar-value-text">
          {formatPopulation(c.population)}
        </span>
      </div>
      <div className="statusbar-separator" />
      <div className="statusbar-item">
        <span className="statusbar-label-text">GDP</span>
        <span className="statusbar-value-text">{formatGdp(c.gdp)}</span>
      </div>
      <div className="statusbar-separator" />
      <div className="statusbar-item">
        <span
          className={`statusbar-growth ${
            c.gdp_growth >= 0 ? 'positive' : 'negative'
          }`}
        >
          {c.gdp_growth >= 0 ? '+' : ''}
          {c.gdp_growth.toFixed(1)}%
        </span>
      </div>
      <div className="statusbar-separator" />
      <div className="statusbar-military">
        <MilitaryBar
          label="Army"
          value={c.military_power.army_strength}
          color="#ef4444"
        />
        <MilitaryBar
          label="Navy"
          value={c.military_power.navy_strength}
          color="#3b82f6"
        />
        <MilitaryBar
          label="Air"
          value={c.military_power.air_force_strength}
          color="#8b5cf6"
        />
      </div>
      <div className="statusbar-separator" />
      <div className="statusbar-item">
        <span className="statusbar-label-text">Stability</span>
        <div className="stability-indicator">
          <div className="stability-track">
            <div
              className="stability-fill"
              style={{ width: `${Math.min(c.stability, 100)}%` }}
            />
          </div>
          <span className="statusbar-value-text">{c.stability.toFixed(0)}%</span>
        </div>
      </div>
      <div className="statusbar-separator" />
      <div className="statusbar-item">
        <span className={`tech-badge tech-badge--${c.tech_level.toLowerCase()}`}>
          {c.tech_level}
        </span>
      </div>
      <div className="statusbar-separator" />
      <div className="statusbar-item">
        <span className="statusbar-label-text">Wars</span>
        <span className="statusbar-value-text">{c.active_wars.length}</span>
      </div>
    </div>
  );
};

export default StatusBar;
