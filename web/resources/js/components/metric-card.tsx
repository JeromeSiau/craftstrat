import { Card, CardContent } from '@/components/ui/card';

interface MetricCardProps {
    label: string;
    value: string | number;
    icon?: React.ComponentType<{ className?: string }>;
    trend?: 'up' | 'down' | 'neutral';
}

export default function MetricCard({ label, value, icon: Icon, trend }: MetricCardProps) {
    return (
        <Card className="relative overflow-hidden">
            <CardContent className="pt-5 pb-5">
                <div className="flex items-start justify-between">
                    <div className="space-y-1">
                        <p className="text-xs font-medium tracking-wide text-muted-foreground uppercase">
                            {label}
                        </p>
                        <p
                            className={`text-3xl font-bold tracking-tight ${
                                trend === 'up'
                                    ? 'text-emerald-600 dark:text-emerald-400'
                                    : trend === 'down'
                                      ? 'text-red-500 dark:text-red-400'
                                      : ''
                            }`}
                        >
                            {value}
                        </p>
                    </div>
                    {Icon && (
                        <div className="rounded-lg bg-primary/10 p-2.5">
                            <Icon className="size-5 text-primary" />
                        </div>
                    )}
                </div>
            </CardContent>
        </Card>
    );
}
