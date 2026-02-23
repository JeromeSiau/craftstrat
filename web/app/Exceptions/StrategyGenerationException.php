<?php

namespace App\Exceptions;

use RuntimeException;

class StrategyGenerationException extends RuntimeException
{
    public static function apiError(int $status): self
    {
        return new self("AI generation failed with status {$status}.");
    }

    public static function invalidJson(string $raw): self
    {
        return new self('AI returned invalid JSON: '.mb_substr($raw, 0, 200));
    }

    public static function validationFailed(string $reason): self
    {
        return new self("Generated strategy is invalid: {$reason}.");
    }
}
