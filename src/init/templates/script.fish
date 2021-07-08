functions -q _original_fish_prompt
if test $status -ne 0
  functions -c fish_prompt _original_fish_prompt
end

function __awscredx_prompt
  @bin@ print-prompt
end

function assume
  set -l output ("@bin@" assume $argv)
  set -l s $status
  switch $s
    case 0
      eval "$output"
      if test "$__awscredx_modify_prompt" = "true"
        function fish_prompt
          set -l old_status $status

          __awscredx_prompt

          echo -n "exit $old_status" | .
          _original_fish_prompt
        end
      end
    case '*'
      return $s
  end
end
