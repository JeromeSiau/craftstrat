<?php

namespace App\Policies;

use App\Models\BacktestResult;
use App\Models\User;

class BacktestResultPolicy
{
    public function view(User $user, BacktestResult $backtestResult): bool
    {
        return $user->id === $backtestResult->user_id;
    }
}
