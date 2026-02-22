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
                </div>
            </CardContent>
        </Card>
    );
}
