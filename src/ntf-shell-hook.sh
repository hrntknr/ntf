AUTO_NTF_DONE_IGNORE=${AUTO_NTF_DONE_IGNORE:-ntf emacs htop info less mail man meld most mutt nano screen ssh tail tmux top vi vim watch}

function _ntf_precmd() {
  local code="$?"
  [[ -n "$ntf_start_time" ]] || return
  local duration=$(($(date +%s) - $ntf_start_time))
  ntf_start_time=''
  [[ "$duration" < "$AUTO_NTF_DONE_LONGER_THAN" ]] && return

  local appname=$(basename "${ntf_command%% *}")
  [[ " $AUTO_NTF_DONE_IGNORE " == *" $appname "* ]] && return

  (ntf shell-done "$code" "$duration" "$ntf_command" &)
}

function _ntf_preexec() {
  ntf_start_time=$(date +%s)
  ntf_command="$1"
}

function _contains_element() {
  local e
  for e in "${@:2}"; do [[ "$e" == "$1" ]] && return 0; done
  return 1
}

if ! _contains_element _ntf_preexec "${preexec_functions[@]}"; then
  preexec_functions+=(_ntf_preexec)
fi

if ! _contains_element _ntf_precmd "${precmd_functions[@]}"; then
  precmd_functions+=(_ntf_precmd)
fi
