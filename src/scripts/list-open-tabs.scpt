tell application "Safari"
    repeat with t in tabs of windows
        -- I'm not sure why I can't just do `log (URL of t)` and get
        -- sensible output.  Going via a variable works better, apparently.
        set theURL to URL of t
        log theURL
    end repeat
end tell
