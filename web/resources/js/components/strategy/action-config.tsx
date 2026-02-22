import { Zap } from 'lucide-react';
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
import { safeParseFloat } from '@/lib/formatters';
import type { StrategyAction } from '@/types/models';

interface ActionConfigProps {
    action: StrategyAction;
    onChange: (action: StrategyAction) => void;
}

export default function ActionConfig({ action, onChange }: ActionConfigProps) {
    return (
        <Card className="border-l-4 border-l-emerald-500/50">
            <CardHeader>
                <div className="flex items-center gap-3">
                    <div className="rounded-lg bg-emerald-500/10 p-2 dark:bg-emerald-500/15">
                        <Zap className="size-4 text-emerald-600 dark:text-emerald-400" />
                    </div>
                    <div>
                        <CardTitle>Action</CardTitle>
                        <p className="text-sm text-muted-foreground">
                            Configure what happens when conditions are met.
                        </p>
                    </div>
                </div>
            </CardHeader>
            <CardContent>
                <div className="grid gap-6 sm:grid-cols-2 xl:grid-cols-3">
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
                                onChange({ ...action, size_usdc: safeParseFloat(e.target.value) })
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
