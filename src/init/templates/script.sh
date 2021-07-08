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
      if [ "$__awscredx_modify_prompt" = "true" ]; then
        if [[ $SHELL =~ zsh ]]; then
          setopt PROMPT_SUBST
        fi
        PS1='$(__awscredx_prompt) '${_ORIGINAL_PS1:-}
      fi
      ;;
    *)
      echo $out
      return $s
      ;;
  esac
}
