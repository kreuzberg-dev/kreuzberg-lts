#!/usr/bin/env bats
#
# Contract tests for scripts/lib/retry.sh.
#
# FAN-OUT SHARED: byte-identical in xberg and kreuzberg-lts, where the script under test is
# md5-identical. crawlberg has no retry.sh, so the suite is simply absent there.
#
# `sleep` is stubbed in every test that reaches the backoff path. The real delays are 5s then
# 10s, so an unstubbed suite would cost 15 seconds per retry case and would assert nothing about
# the schedule. The stub records what was asked for, which turns the delay into something the
# tests can check rather than something they wait out. ~keep

setup() {
	bats_load_library xberg-bats
	xberg_setup
	LIB="$(cd "$BATS_TEST_DIRNAME/../../.." && pwd -P)/scripts/lib/retry.sh"

	# A command whose per-attempt outcome the test controls: it appends to a counter file and
	# exits with the status named for that attempt.
	ATTEMPTS="$XBERG_WORK/attempts"
	: >"$ATTEMPTS"
}

# Build a command that fails for the first N invocations and then succeeds.
make_flaky() {
	local fail_count="$1"
	xberg_stub flaky \
		"printf 'x' >>\"\$ATTEMPTS\"" \
		"count=\$(wc -c <\"\$ATTEMPTS\" | tr -d ' ')" \
		"[ \"\$count\" -gt $fail_count ]"
}

attempt_count() {
	wc -c <"$ATTEMPTS" | tr -d ' '
}

# --- retry_with_backoff -------------------------------------------------------------------------

@test "retry_with_backoff should_run_the_command_once_when_it_succeeds_immediately" {
	make_flaky 0
	xberg_stub_trace sleep

	run bash -c 'source "$1"; ATTEMPTS="$2" retry_with_backoff flaky' _ "$LIB" "$ATTEMPTS"

	xberg_assert_status 0
	[ "$(attempt_count)" -eq 1 ]
	xberg_assert_trace_empty
}

@test "retry_with_backoff should_succeed_on_the_third_attempt_when_the_first_two_fail" {
	make_flaky 2
	xberg_stub_trace sleep

	run bash -c 'source "$1"; ATTEMPTS="$2" XBERG_TRACE="$3" retry_with_backoff flaky' \
		_ "$LIB" "$ATTEMPTS" "$XBERG_TRACE"

	xberg_assert_status 0
	[ "$(attempt_count)" -eq 3 ]
}

@test "retry_with_backoff should_give_up_after_three_attempts_when_the_command_never_succeeds" {
	make_flaky 99
	xberg_stub_trace sleep

	run bash -c 'source "$1"; ATTEMPTS="$2" XBERG_TRACE="$3" retry_with_backoff flaky' \
		_ "$LIB" "$ATTEMPTS" "$XBERG_TRACE"

	xberg_assert_status 1
	[ "$(attempt_count)" -eq 3 ]
}

@test "retry_with_backoff should_double_the_delay_between_attempts_when_it_retries" {
	make_flaky 99
	xberg_stub_trace sleep

	run bash -c 'source "$1"; ATTEMPTS="$2" XBERG_TRACE="$3" retry_with_backoff flaky' \
		_ "$LIB" "$ATTEMPTS" "$XBERG_TRACE"

	# Two sleeps for three attempts, doubling: the last failure is not followed by a wait.
	xberg_assert_trace "sleep 5" "sleep 10"
}

@test "retry_with_backoff should_not_sleep_after_the_final_failure" {
	make_flaky 99
	xberg_stub_trace sleep

	run bash -c 'source "$1"; ATTEMPTS="$2" XBERG_TRACE="$3" retry_with_backoff flaky' \
		_ "$LIB" "$ATTEMPTS" "$XBERG_TRACE"

	[ "$(grep -c . "$XBERG_TRACE")" -eq 2 ]
}

# --- run_with_timeout ---------------------------------------------------------------------------

@test "run_with_timeout should_delegate_to_timeout_when_it_is_available" {
	xberg_stub_trace timeout
	run bash -c 'source "$1"; XBERG_TRACE="$2" run_with_timeout 7 echo hello' _ "$LIB" "$XBERG_TRACE"

	xberg_assert_status 0
	xberg_assert_trace "timeout 7 echo hello"
}

@test "run_with_timeout should_propagate_the_status_of_the_command_it_wraps" {
	xberg_stub timeout 'shift; "$@"'
	run bash -c 'source "$1"; run_with_timeout 7 sh -c "exit 9"' _ "$LIB"
	xberg_assert_status 9
}
