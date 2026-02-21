<?php

namespace Database\Factories;

use App\Models\User;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends \Illuminate\Database\Eloquent\Factories\Factory<\App\Models\Wallet>
 */
class WalletFactory extends Factory
{
    /**
     * Define the model's default state.
     *
     * @return array<string, mixed>
     */
    public function definition(): array
    {
        return [
            'user_id' => User::factory(),
            'label' => fake()->optional()->words(2, true),
            'address' => '0x'.fake()->regexify('[a-fA-F0-9]{40}'),
            'private_key_enc' => base64_encode(fake()->sha256()),
            'balance_usdc' => fake()->randomFloat(6, 0, 10000),
            'is_active' => true,
        ];
    }

    /**
     * Indicate that the wallet is inactive.
     */
    public function inactive(): static
    {
        return $this->state(fn () => ['is_active' => false]);
    }
}
