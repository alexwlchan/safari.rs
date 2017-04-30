set listOfUrls to {}

tell application "Safari"
  repeat with t in tabs of windows
    copy (URL of t) to the end of listOfUrls
  end repeat
end tell

get listOfUrls
