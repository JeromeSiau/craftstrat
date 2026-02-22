interface StatusBadgeProps {
    active: boolean;
}

export default function StatusBadge({ active }: StatusBadgeProps) {
    return (
        <span
            className={`inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-semibold ${
                active
                    ? 'bg-emerald-500/10 text-emerald-700 dark:bg-emerald-500/15 dark:text-emerald-400'
                    : 'bg-muted text-muted-foreground'
            }`}
        >
            <span
                className={`size-1.5 rounded-full ${
                    active
                        ? 'bg-emerald-500 shadow-[0_0_4px_1px] shadow-emerald-500/40'
                        : 'bg-muted-foreground/40'
                }`}
            />
            {active ? 'Active' : 'Inactive'}
        </span>
    );
}
