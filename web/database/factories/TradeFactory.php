<?php

namespace Database\Factories;

use App\Models\Strategy;
use App\Models\Wallet;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends \Illuminate\Database\Eloquent\Factories\Factory<\App\Models\Trade>
 */
class TradeFactory extends Factory
{
    /**
     * Define the model's default state.
     *
     * @return array<string, mixed>
     */
    public function definition(): array
    {
        return [
            'wallet_id' => Wallet::factory(),
            'strategy_id' => Strategy::factory(),
            'market_id' => 'btc-updown-15m-'.fake()->unixTime(),
            'side' => fake()->randomElement(['buy', 'sell']),
            'outcome' => fake()->randomElement(['UP', 'DOWN']),
            'price' => fake()->randomFloat(6, 0.01, 0.99),
            'size_usdc' => fake()->randomFloat(6, 10, 500),
            'order_type' => fake()->randomElement(['market', 'limit']),
            'status' => 'filled',
            'fee_bps' => fake()->randomElement([0, 50, 100]),
            'executed_at' => now(),
        ];
    }

    /**
     * Indicate that the trade is pending.
     */
    public function pending(): static
    {
        return $this->state(fn () => ['status' => 'pending', 'executed_at' => null]);
    }
}
