import { Card, CardContent } from '@/components/ui/card';

type AccentColor = 'blue' | 'emerald' | 'amber' | 'violet' | 'red';

const accentStyles: Record<AccentColor, { icon: string; bg: string }> = {
    blue: { icon: 'text-blue-600 dark:text-blue-400', bg: 'bg-blue-500/10 dark:bg-blue-400/10' },
    emerald: { icon: 'text-emerald-600 dark:text-emerald-400', bg: 'bg-emerald-500/10 dark:bg-emerald-400/10' },
    amber: { icon: 'text-amber-600 dark:text-amber-400', bg: 'bg-amber-500/10 dark:bg-amber-400/10' },
    violet: { icon: 'text-violet-600 dark:text-violet-400', bg: 'bg-violet-500/10 dark:bg-violet-400/10' },
    red: { icon: 'text-red-600 dark:text-red-400', bg: 'bg-red-500/10 dark:bg-red-400/10' },
};

interface MetricCardProps {
    label: string;
    value: string | number;
    icon?: React.ComponentType<{ className?: string }>;
    trend?: 'up' | 'down' | 'neutral';
    accent?: AccentColor;
}

export default function MetricCard({ label, value, icon: Icon, trend, accent }: MetricCardProps) {
    const colors = accent ? accentStyles[accent] : null;

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
                        <div className={`rounded-lg p-2.5 ${colors?.bg ?? 'bg-primary/10'}`}>
                            <Icon className={`size-5 ${colors?.icon ?? 'text-primary'}`} />
                        </div>
                    )}
                </div>
            </CardContent>
        </Card>
    );
}
