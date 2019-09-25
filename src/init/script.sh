export AWSCREDX_SCRIPT_VERSION="@version@"
function assume {
  out=$("@bin@" assume "$@")
  if [[ $? == 0 ]]; then
    $($out)
  fi
}
