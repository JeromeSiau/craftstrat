<?php

namespace App\Notifications;

use Illuminate\Bus\Queueable;
use Illuminate\Notifications\Messages\MailMessage;
use Illuminate\Notifications\Notification;

class StrategyAlertNotification extends Notification
{
    use Queueable;

    public function __construct(
        public string $strategyName,
        public string $message,
        public string $channel = 'database',
    ) {}

    /**
     * @return array<int, string>
     */
    public function via(object $notifiable): array
    {
        return match ($this->channel) {
            'mail' => ['mail', 'database'],
            default => ['database'],
        };
    }

    public function toMail(object $notifiable): MailMessage
    {
        return (new MailMessage)
            ->subject("Strategy Alert: {$this->strategyName}")
            ->line($this->message)
            ->action('View Dashboard', url('/dashboard'));
    }

    /**
     * @return array<string, mixed>
     */
    public function toArray(object $notifiable): array
    {
        return [
            'strategy_name' => $this->strategyName,
            'message' => $this->message,
        ];
    }
}
