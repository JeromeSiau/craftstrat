import { ShieldAlert } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { safeParseFloat } from '@/lib/formatters';
import type { StrategyRisk } from '@/types/models';

interface RiskConfigProps {
    risk: StrategyRisk;
    onChange: (risk: StrategyRisk) => void;
}

export default function RiskConfig({ risk, onChange }: RiskConfigProps) {
    function handleChange(field: 'max_position_usdc' | 'max_trades_per_slot', value: string): void {
        onChange({ ...risk, [field]: safeParseFloat(value) });
    }

    function handleToggleSL(checked: boolean): void {
        onChange({ ...risk, stoploss_pct: checked ? 30 : null });
    }

    function handleToggleTP(checked: boolean): void {
        onChange({ ...risk, take_profit_pct: checked ? 80 : null });
    }

    function handleToggleDailyLoss(checked: boolean): void {
        onChange({ ...risk, daily_loss_limit_usdc: checked ? 100 : null });
    }

    function handleToggleCooldown(checked: boolean): void {
        onChange({ ...risk, cooldown_seconds: checked ? 60 : null });
    }

    function handleToggleDuplicates(checked: boolean): void {
        onChange({ ...risk, prevent_duplicates: checked });
    }

    return (
        <Card className="border-l-4 border-l-amber-500/50">
            <CardHeader>
                <div className="flex items-center gap-3">
                    <div className="rounded-lg bg-amber-500/10 p-2 dark:bg-amber-500/15">
                        <ShieldAlert className="size-4 text-amber-600 dark:text-amber-400" />
                    </div>
                    <div>
                        <CardTitle>Risk Management</CardTitle>
                        <p className="text-sm text-muted-foreground">
                            Set limits to protect your positions.
                        </p>
                    </div>
                </div>
            </CardHeader>
            <CardContent>
                <div className="grid gap-6 sm:grid-cols-2 xl:grid-cols-4">
                    <div className="space-y-2">
                        <Label htmlFor="max_position_usdc">Max Position (USDC)</Label>
                        <Input
                            id="max_position_usdc"
                            type="number"
                            min={1}
                            step="any"
                            value={risk.max_position_usdc}
                            onChange={(e) => handleChange('max_position_usdc', e.target.value)}
                        />
                    </div>

                    <div className="space-y-2">
                        <Label htmlFor="max_trades_per_slot">Max Trades / Slot</Label>
                        <Input
                            id="max_trades_per_slot"
                            type="number"
                            min={1}
                            step={1}
                            value={risk.max_trades_per_slot}
                            onChange={(e) => handleChange('max_trades_per_slot', e.target.value)}
                        />
                    </div>

                    <div className="space-y-3">
                        <div className="flex items-center gap-2">
                            <Checkbox
                                id="stoploss_enabled"
                                checked={risk.stoploss_pct !== null}
                                onCheckedChange={handleToggleSL}
                            />
                            <Label htmlFor="stoploss_enabled">Stop Loss (%)</Label>
                        </div>
                        {risk.stoploss_pct !== null && (
                            <Input
                                id="stoploss_pct"
                                type="number"
                                min={0}
                                max={100}
                                step="any"
                                value={risk.stoploss_pct}
                                onChange={(e) =>
                                    onChange({ ...risk, stoploss_pct: safeParseFloat(e.target.value) })
                                }
                            />
                        )}
                    </div>

                    <div className="space-y-3">
                        <div className="flex items-center gap-2">
                            <Checkbox
                                id="take_profit_enabled"
                                checked={risk.take_profit_pct !== null}
                                onCheckedChange={handleToggleTP}
                            />
                            <Label htmlFor="take_profit_enabled">Take Profit (%)</Label>
                        </div>
                        {risk.take_profit_pct !== null && (
                            <Input
                                id="take_profit_pct"
                                type="number"
                                min={0}
                                max={100}
                                step="any"
                                value={risk.take_profit_pct}
                                onChange={(e) =>
                                    onChange({ ...risk, take_profit_pct: safeParseFloat(e.target.value) })
                                }
                            />
                        )}
                    </div>

                    <div className="space-y-3">
                        <div className="flex items-center gap-2">
                            <Checkbox
                                id="daily_loss_enabled"
                                checked={risk.daily_loss_limit_usdc !== null}
                                onCheckedChange={handleToggleDailyLoss}
                            />
                            <Label htmlFor="daily_loss_enabled">Daily Loss Limit (USDC)</Label>
                        </div>
                        {risk.daily_loss_limit_usdc !== null && (
                            <Input
                                id="daily_loss_limit_usdc"
                                type="number"
                                min={1}
                                step="any"
                                value={risk.daily_loss_limit_usdc}
                                onChange={(e) =>
                                    onChange({ ...risk, daily_loss_limit_usdc: safeParseFloat(e.target.value) })
                                }
                            />
                        )}
                    </div>

                    <div className="space-y-3">
                        <div className="flex items-center gap-2">
                            <Checkbox
                                id="cooldown_enabled"
                                checked={risk.cooldown_seconds !== null}
                                onCheckedChange={handleToggleCooldown}
                            />
                            <Label htmlFor="cooldown_enabled">Cooldown (seconds)</Label>
                        </div>
                        {risk.cooldown_seconds !== null && (
                            <Input
                                id="cooldown_seconds"
                                type="number"
                                min={1}
                                step={1}
                                value={risk.cooldown_seconds}
                                onChange={(e) =>
                                    onChange({ ...risk, cooldown_seconds: safeParseFloat(e.target.value) })
                                }
                            />
                        )}
                    </div>

                    <div className="space-y-3">
                        <div className="flex items-center gap-2">
                            <Checkbox
                                id="prevent_duplicates"
                                checked={risk.prevent_duplicates}
                                onCheckedChange={handleToggleDuplicates}
                            />
                            <Label htmlFor="prevent_duplicates">Prevent Duplicates</Label>
                        </div>
                    </div>
                </div>
            </CardContent>
        </Card>
    );
}
