+++
type = "sr-test"
title = "Overdue Item"
status = "completed"
review_type = "recall"

[sr]
next_review = "2026-01-01"
interval = 6
ease = 2.50
reps = 2
last_reviewed = "2025-12-26"
+++

# Overdue SR Test Fixture

This item is overdue — next_review is in the past.
Used to verify `today`, `review`, `stats`, and SR query tests.
