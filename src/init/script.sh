# This file is generated by @bin@.
# Do not edit.

export AWSCREDX_SCRIPT_VERSION="@version@"
_ORIGINAL_PS1="${PS1:-}"
function assume {
  out=$("@bin@" assume "$@")
  s=$?
  if [[ $s == 0 ]]; then
    $out
    PS1="[\e[1;35m\$AWS_PROFILE\e[m] ${_ORIGINAL_PS1:-}"
  else
    return $s
  fi
}
