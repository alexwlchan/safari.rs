tell application "Safari"
  repeat with t in tabs of windows
    tell t
      -- If you open lots of windows in Safari, some of this book-
      -- keeping goes wrong.  It will try to look up tab N, except
      -- tab N was already closed -- error!
      --
      -- For safety, we just catch and discard all errors.
      {% for condition in conditions %}
      try
        tell t
          if (URL {{ condition}}) then close
        end tell
      end try
      {% endfor %}
    end tell
  end repeat
end tell
