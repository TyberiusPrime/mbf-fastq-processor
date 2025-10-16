Outcome: Need prodding to get to the right implementation
# CalcRate step

## Goal
Implement the CalcRate transformation that derives a numeric tag representing a rate (optionally log-scaled) from existing numeric tags, following dev/issues/0043.

## Plan
1. Review calc step architecture (enum wiring, module layout, config validation) and decide on the shape of CalcRate inputs, including segment handling for the implicit denominator.
2. Implement `CalcRate` in `src/transformations/calc/`, covering validation of upstream tags, per-read rate computation (with length fallback), log options, and enum/template wiring.
3. Update documentation (reference docs + template) to describe CalcRate fields and behaviour.
4. Add integration fixtures under `test_cases/calc/calc_rate/` exercising default, explicit denominator, and log variants; regenerate test harness and run targeted cargo tests.
