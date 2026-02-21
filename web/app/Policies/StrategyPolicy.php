<?php

namespace App\Policies;

use App\Models\Strategy;
use App\Models\User;

class StrategyPolicy
{
    public function view(User $user, Strategy $strategy): bool
    {
        return $user->id === $strategy->user_id;
    }

    public function update(User $user, Strategy $strategy): bool
    {
        return $user->id === $strategy->user_id;
    }

    public function delete(User $user, Strategy $strategy): bool
    {
        return $user->id === $strategy->user_id;
    }
}
