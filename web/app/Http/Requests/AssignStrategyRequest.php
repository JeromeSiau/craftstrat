<?php

namespace App\Http\Requests;

use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;

class AssignStrategyRequest extends FormRequest
{
    /**
     * Get the validation rules that apply to the request.
     *
     * @return array<string, ValidationRule|array<mixed>|string>
     */
    public function rules(): array
    {
        return [
            'strategy_id' => ['required', 'exists:strategies,id'],
            'markets' => ['nullable', 'array'],
            'markets.*' => ['string'],
            'max_position_usdc' => ['nullable', 'numeric', 'min:1', 'max:1000000'],
        ];
    }
}
