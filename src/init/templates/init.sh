export @shell_var@=@shell@
if [ -z "$_ORIGINAL_PS1" ]; then
  _ORIGINAL_PS1="${PS1:-}"
fi

function __awscredx_prompt {
  @bin@ print-prompt
}

function assume {
  out=$("@bin@" assume "$@")
  s=$?
  case $s in
    0)
      eval $out
      ;;
    *)
      return $s
      ;;
  esac
}
