import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import type { StrategyRisk } from '@/types/models';

interface RiskConfigProps {
    risk: StrategyRisk;
    onChange: (risk: StrategyRisk) => void;
}

export default function RiskConfig({ risk, onChange }: RiskConfigProps) {
    function handleChange(field: keyof StrategyRisk, value: string): void {
        const numValue = value === '' ? 0 : parseFloat(value);
        onChange({ ...risk, [field]: isNaN(numValue) ? 0 : numValue });
    }

    return (
        <Card>
            <CardHeader>
                <CardTitle className="text-sm">Risk Management</CardTitle>
            </CardHeader>
            <CardContent>
                <div className="grid gap-4 sm:grid-cols-2">
                    <div className="space-y-2">
                        <Label htmlFor="stoploss_pct">Stop Loss (%)</Label>
                        <Input
                            id="stoploss_pct"
                            type="number"
                            min={0}
                            max={100}
                            step="any"
                            value={risk.stoploss_pct}
                            onChange={(e) => handleChange('stoploss_pct', e.target.value)}
                        />
                    </div>

                    <div className="space-y-2">
                        <Label htmlFor="take_profit_pct">Take Profit (%)</Label>
                        <Input
                            id="take_profit_pct"
                            type="number"
                            min={0}
                            max={100}
                            step="any"
                            value={risk.take_profit_pct}
                            onChange={(e) => handleChange('take_profit_pct', e.target.value)}
                        />
                    </div>

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
                </div>
            </CardContent>
        </Card>
    );
}
