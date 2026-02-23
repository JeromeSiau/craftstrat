import { indicators } from '@/components/strategy/indicator-options';
import type { FormModeGraph, ConditionGroup, StrategyRule } from '@/types/models';

const indicatorLabelMap = Object.fromEntries(
    indicators.map((i) => [i.value, i.label]),
);

function formatRuleValue(rule: StrategyRule): string {
    if (rule.operator === 'between' && Array.isArray(rule.value)) {
        return `${rule.value[0]} and ${rule.value[1]}`;
    }
    return String(rule.value);
}

export function isFormModeGraph(graph: Record<string, unknown>): graph is FormModeGraph {
    return graph.mode === 'form' && Array.isArray(graph.conditions);
}

export default function StrategyRulesDisplay({ graph }: { graph: FormModeGraph }) {
    return (
        <div className="mt-4 space-y-4">
            <h3 className="text-sm font-semibold">Strategy Rules</h3>

            {graph.conditions.map((group: ConditionGroup, groupIndex: number) => (
                <div key={group.id ?? groupIndex} className="rounded-md border p-3">
                    <p className="mb-2 text-xs font-medium text-muted-foreground">
                        Condition Group {groupIndex + 1} ({group.type})
                    </p>
                    <ul className="space-y-1 text-sm">
                        {group.rules.map((rule: StrategyRule, ruleIndex: number) => (
                            <li key={rule.id ?? ruleIndex} className="flex items-center gap-1.5">
                                <span className="font-medium">
                                    {indicatorLabelMap[rule.indicator] || rule.indicator}
                                </span>
                                <span className="text-muted-foreground">{rule.operator}</span>
                                <span>{formatRuleValue(rule)}</span>
                            </li>
                        ))}
                    </ul>
                </div>
            ))}

            <div className="rounded-md border p-3">
                <p className="mb-2 text-xs font-medium text-muted-foreground">Action</p>
                <dl className="grid grid-cols-2 gap-1 text-sm">
                    <dt className="text-muted-foreground">Signal</dt>
                    <dd className="capitalize">{graph.action.signal}</dd>
                    <dt className="text-muted-foreground">Outcome</dt>
                    <dd>{graph.action.outcome}</dd>
                    <dt className="text-muted-foreground">Size</dt>
                    <dd>{graph.action.size_usdc} USDC</dd>
                    <dt className="text-muted-foreground">Order Type</dt>
                    <dd className="capitalize">{graph.action.order_type}</dd>
                </dl>
            </div>

            <div className="rounded-md border p-3">
                <p className="mb-2 text-xs font-medium text-muted-foreground">Risk</p>
                <dl className="grid grid-cols-2 gap-1 text-sm">
                    <dt className="text-muted-foreground">Stop Loss</dt>
                    <dd>{graph.risk.stoploss_pct !== null ? `${graph.risk.stoploss_pct}%` : 'Off'}</dd>
                    <dt className="text-muted-foreground">Take Profit</dt>
                    <dd>{graph.risk.take_profit_pct !== null ? `${graph.risk.take_profit_pct}%` : 'Off'}</dd>
                    <dt className="text-muted-foreground">Max Position</dt>
                    <dd>{graph.risk.max_position_usdc} USDC</dd>
                    <dt className="text-muted-foreground">Max Trades / Slot</dt>
                    <dd>{graph.risk.max_trades_per_slot}</dd>
                    <dt className="text-muted-foreground">Daily Loss Limit</dt>
                    <dd>{graph.risk.daily_loss_limit_usdc !== null ? `${graph.risk.daily_loss_limit_usdc} USDC` : 'Off'}</dd>
                    <dt className="text-muted-foreground">Cooldown</dt>
                    <dd>{graph.risk.cooldown_seconds !== null ? `${graph.risk.cooldown_seconds}s` : 'Off'}</dd>
                    <dt className="text-muted-foreground">Duplicate Prevention</dt>
                    <dd>{graph.risk.prevent_duplicates ? 'On' : 'Off'}</dd>
                </dl>
            </div>
        </div>
    );
}
