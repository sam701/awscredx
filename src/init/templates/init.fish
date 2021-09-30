set -x @shell_var@ fish
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
    case '*'
      return $s
  end
end
