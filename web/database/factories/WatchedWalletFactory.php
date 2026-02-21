<?php

namespace Database\Factories;

use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends \Illuminate\Database\Eloquent\Factories\Factory<\App\Models\WatchedWallet>
 */
class WatchedWalletFactory extends Factory
{
    /**
     * Define the model's default state.
     *
     * @return array<string, mixed>
     */
    public function definition(): array
    {
        return [
            'address' => '0x'.fake()->regexify('[a-fA-F0-9]{40}'),
            'label' => fake()->optional()->words(2, true),
            'follower_count' => fake()->numberBetween(0, 100),
            'win_rate' => fake()->optional()->randomFloat(4, 0.3, 0.8),
            'total_pnl_usdc' => fake()->optional()->randomFloat(6, -1000, 10000),
            'avg_slippage' => fake()->optional()->randomFloat(6, 0, 0.05),
            'last_seen_at' => fake()->optional()->dateTimeThisMonth(),
        ];
    }
}
