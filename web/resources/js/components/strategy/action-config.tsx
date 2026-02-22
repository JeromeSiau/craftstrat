import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';
import type { StrategyAction } from '@/types/models';

interface ActionConfigProps {
    action: StrategyAction;
    onChange: (action: StrategyAction) => void;
}

export default function ActionConfig({ action, onChange }: ActionConfigProps) {
    return (
        <Card>
            <CardHeader>
                <CardTitle className="text-sm">Action (ALORS)</CardTitle>
            </CardHeader>
            <CardContent>
                <div className="grid gap-4 sm:grid-cols-2">
                    <div className="space-y-2">
                        <Label htmlFor="signal">Signal</Label>
                        <Select
                            value={action.signal}
                            onValueChange={(value) =>
                                onChange({ ...action, signal: value as 'buy' | 'sell' })
                            }
                        >
                            <SelectTrigger id="signal">
                                <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="buy">Buy</SelectItem>
                                <SelectItem value="sell">Sell</SelectItem>
                            </SelectContent>
                        </Select>
                    </div>

                    <div className="space-y-2">
                        <Label htmlFor="outcome">Outcome</Label>
                        <Select
                            value={action.outcome}
                            onValueChange={(value) =>
                                onChange({ ...action, outcome: value as 'UP' | 'DOWN' })
                            }
                        >
                            <SelectTrigger id="outcome">
                                <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="UP">UP</SelectItem>
                                <SelectItem value="DOWN">DOWN</SelectItem>
                            </SelectContent>
                        </Select>
                    </div>

                    <div className="space-y-2">
                        <Label htmlFor="size_mode">Size Mode</Label>
                        <Select
                            value={action.size_mode}
                            onValueChange={(value) =>
                                onChange({ ...action, size_mode: value as 'fixed' | 'proportional' })
                            }
                        >
                            <SelectTrigger id="size_mode">
                                <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="fixed">Fixed</SelectItem>
                                <SelectItem value="proportional">Proportional</SelectItem>
                            </SelectContent>
                        </Select>
                    </div>

                    <div className="space-y-2">
                        <Label htmlFor="size_usdc">Size (USDC)</Label>
                        <Input
                            id="size_usdc"
                            type="number"
                            min={1}
                            step="any"
                            value={action.size_usdc}
                            onChange={(e) =>
                                onChange({
                                    ...action,
                                    size_usdc: e.target.value === '' ? 0 : parseFloat(e.target.value),
                                })
                            }
                        />
                    </div>

                    <div className="space-y-2">
                        <Label htmlFor="order_type">Order Type</Label>
                        <Select
                            value={action.order_type}
                            onValueChange={(value) =>
                                onChange({ ...action, order_type: value as 'market' | 'limit' })
                            }
                        >
                            <SelectTrigger id="order_type">
                                <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="market">Market</SelectItem>
                                <SelectItem value="limit">Limit</SelectItem>
                            </SelectContent>
                        </Select>
                    </div>
                </div>
            </CardContent>
        </Card>
    );
}
