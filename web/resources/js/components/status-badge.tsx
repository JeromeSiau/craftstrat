interface StatusBadgeProps {
    active: boolean;
}

export default function StatusBadge({ active }: StatusBadgeProps) {
    return (
        <span
            className={`rounded-full px-2 py-1 text-xs font-medium ${
                active
                    ? 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300'
                    : 'bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400'
            }`}
        >
            {active ? 'Active' : 'Inactive'}
        </span>
    );
}
