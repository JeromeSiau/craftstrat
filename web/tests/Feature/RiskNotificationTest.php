<?php

use App\Models\User;
use App\Models\Wallet;
use App\Notifications\StrategyAlertNotification;
use Illuminate\Support\Facades\Notification;

beforeEach(function () {
    $this->user = User::factory()->create();
});

it('sends database notification via internal endpoint', function () {
    Notification::fake();

    $wallet = Wallet::factory()->create(['user_id' => $this->user->id]);

    $this->postJson(route('internal.notification.send'), [
        'wallet_id' => $wallet->id,
        'strategy_name' => 'My Strategy',
        'message' => 'Daily loss limit reached',
        'channel' => 'database',
    ])->assertOk()
        ->assertJson(['status' => 'sent']);

    Notification::assertSentTo($this->user, StrategyAlertNotification::class, function ($notification) {
        return $notification->strategyName === 'My Strategy'
            && $notification->message === 'Daily loss limit reached'
            && $notification->channel === 'database';
    });
});

it('sends mail notification via internal endpoint', function () {
    Notification::fake();

    $wallet = Wallet::factory()->create(['user_id' => $this->user->id]);

    $this->postJson(route('internal.notification.send'), [
        'wallet_id' => $wallet->id,
        'strategy_name' => 'EMA Cross',
        'message' => 'Big move detected!',
        'channel' => 'mail',
    ])->assertOk();

    Notification::assertSentTo($this->user, StrategyAlertNotification::class, function ($notification) {
        return $notification->channel === 'mail';
    });
});

it('returns 404 for unknown wallet', function () {
    $this->postJson(route('internal.notification.send'), [
        'wallet_id' => 99999,
        'strategy_name' => 'Test',
        'message' => 'Alert',
        'channel' => 'database',
    ])->assertNotFound();
});

it('validates required fields', function () {
    $this->postJson(route('internal.notification.send'), [])
        ->assertUnprocessable()
        ->assertJsonValidationErrors(['wallet_id', 'strategy_name', 'message', 'channel']);
});

it('validates channel is database or mail', function () {
    $wallet = Wallet::factory()->create(['user_id' => $this->user->id]);

    $this->postJson(route('internal.notification.send'), [
        'wallet_id' => $wallet->id,
        'strategy_name' => 'Test',
        'message' => 'Alert',
        'channel' => 'sms',
    ])->assertUnprocessable()
        ->assertJsonValidationErrors(['channel']);
});

it('notification uses correct channels for database', function () {
    $notification = new StrategyAlertNotification('Test', 'Alert', 'database');

    expect($notification->via($this->user))->toBe(['database']);
});

it('notification uses correct channels for mail', function () {
    $notification = new StrategyAlertNotification('Test', 'Alert', 'mail');

    expect($notification->via($this->user))->toBe(['mail', 'database']);
});

it('notification toArray returns correct data', function () {
    $notification = new StrategyAlertNotification('EMA Cross', 'Price alert!', 'database');

    expect($notification->toArray($this->user))->toBe([
        'strategy_name' => 'EMA Cross',
        'message' => 'Price alert!',
    ]);
});
