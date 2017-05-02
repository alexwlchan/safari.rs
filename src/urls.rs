use urlencoding::{encode as urlencode};
use urlparse::{Query, parse_qs, urlparse, urlunparse};


fn partial_urlencode(value: &str) -> String {
  // The urlencode from the urlencoding library goes further than I like,
  // and also URL encodes ASCII digits. Reverse that stuff.
  urlencode(value)
    .replace("%30", "0")
    .replace("%31", "1")
    .replace("%32", "2")
    .replace("%33", "3")
    .replace("%34", "4")
    .replace("%35", "5")
    .replace("%36", "6")
    .replace("%37", "7")
    .replace("%38", "8")
    .replace("%39", "9")
}


/// Re-encode a query string for Rust
fn encode_querystring(query: Query) -> Option<String> {
  let mut query_components: Vec<String> = vec![];
  for (key, value) in query {
    for v in value.iter() {
      query_components.push(format!("{}={}", key, partial_urlencode(v)));
    }
  }
  if query_components.len() > 0 {
    Some(query_components.join("&"))
  } else {
    None
  }
}


/// Strip tracking junk and URL suffixes.
pub fn tidy_url(url: &str) -> String {
    let mut parsed_url = urlparse(url);

    // Always get the desktop version of Twitter URLs
    if parsed_url.netloc == "mobile.twitter.com" {
        parsed_url.netloc = String::from("twitter.com");
    }

    // Remove any tracking junk from Amazon URLs so they're not a
    // ridiculous length
    if parsed_url.netloc == "www.amazon.co.uk" {
        parsed_url.query = None;
        let mut new_path = String::from("");
        for component in parsed_url.path.split("ref=") {
            new_path = String::from(component);
            break;
        }
        parsed_url.path = new_path;
    }

    // Strip tracking junk from Medium, Mashable and Buzzfeed
    if (parsed_url.netloc == "medium.com") ||
       (parsed_url.netloc == "www.buzzfeed.com") ||
       (parsed_url.netloc == "mashable.com") {
            parsed_url.fragment = None;
    }

    // Remove '#notes' from Tumblr URLs
    if parsed_url.netloc.ends_with("tumblr.com") {
        let new_fragment: Option<String> = match parsed_url.fragment {
            Some(fragment) => {
                if fragment == "notes" {
                    None
                } else {
                    Some(fragment)
                }
            }
            None => None,
        };
        parsed_url.fragment = new_fragment;
    }

  // Remove &feature=youtu.be from YouTube URLs
  if parsed_url.netloc.ends_with("youtube.com") {
    parsed_url.query = match parsed_url.query {
      Some(qs) => {
        let mut query = parse_qs(&qs);
        query.remove("feature");
        encode_querystring(query)
      },
      None => None,
    };
  }

  // Remove any UTM tracking parameters from URLs
  parsed_url.query = match parsed_url.query {
    Some(qs) => {
      let mut query = parse_qs(&qs);
      let utm_keys: Vec<_> = query
        .keys()
        .filter(|key| key.starts_with("utm_"))
        .map(|k| k.clone())
        .collect();
      for key in utm_keys {
        query.remove(&key);
      }
      encode_querystring(query)
    },
    None => None
  };

  urlunparse(parsed_url)
}


macro_rules! tidy_url_tests {
    ($($name:ident: $value:expr,)*) => {
    $(
        #[test]
        fn $name() {
            let (input, expected) = $value;
            assert_eq!(expected, tidy_url(input));
        }
    )*
    }
}


tidy_url_tests! {
  twitter_mobile: (
    "https://mobile.twitter.com/Breaking911/status/822589852596191235",
    "https://twitter.com/Breaking911/status/822589852596191235"
  ),

  regular_twitter: (
    "https://twitter.com/WholesomeMeme/status/846421658835537921",
    "https://twitter.com/WholesomeMeme/status/846421658835537921"
  ),

  amazon_product: (
    "https://www.amazon.co.uk/dp/B01DFKBL68/ref=gw_aucc_comb_AB_clean_2?pf_rd_r=N1MF2PWADRHS55427ETG&pf_rd_p=c11a2c11-c670-46ff-87a1-c1eef4fad652",
    "https://www.amazon.co.uk/dp/B01DFKBL68/"
  ),

  medium_with_tracking: (
    "https://medium.com/@anildash/forget-why-its-time-to-get-to-work-c49ac5f0da20#.sjyskxdsz",
    "https://medium.com/@anildash/forget-why-its-time-to-get-to-work-c49ac5f0da20"
  ),

  tumblr_with_notes: (
    "http://azurelunatic.tumblr.com/post/155525051123/things-about-hufflepuffs-539#notes",
    "http://azurelunatic.tumblr.com/post/155525051123/things-about-hufflepuffs-539"
  ),

  buzzfeed_with_tracking: (
    "https://www.buzzfeed.com/katienotopoulos/the-end-of-apple-man#.biqmkzz8kn",
    "https://www.buzzfeed.com/katienotopoulos/the-end-of-apple-man"
  ),

  mashable_with_tracking: (
    "http://mashable.com/2016/03/21/apple-liam-recycling-robot/#b9y4lv3m4qqX",
    "http://mashable.com/2016/03/21/apple-liam-recycling-robot/"
  ),

  youtube_plain_video: (
    "https://www.youtube.com/watch?v=zB4I68XVPzQ",
    "https://www.youtube.com/watch?v=zB4I68XVPzQ",
  ),

  youtube_with_timestamp: (
    "https://www.youtube.com/watch?v=D68cUzqcTrg&feature=youtu.be#t=1m",
    "https://www.youtube.com/watch?v=D68cUzqcTrg#t=1m",
  ),

  youtube_with_trailing_feature: (
    "https://www.youtube.com/watch?v=PbJqNa0_Oz0&feature=youtu.be",
    "https://www.youtube.com/watch?v=PbJqNa0_Oz0",
  ),

  youtube_with_leading_feature: (
    "https://www.youtube.com/watch?feature=youtu.be&v=oPo4n9tBPsk",
    "https://www.youtube.com/watch?v=oPo4n9tBPsk",
  ),

  youtube_with_only_feature: (
    "https://www.youtube.com/watch?feature=youtu.be",
    "https://www.youtube.com/watch",
  ),

  youtube_channel: (
    "https://www.youtube.com/user/TheQIElves",
    "https://www.youtube.com/user/TheQIElves"
  ),

  single_utm_tracker: (
    "https://example.com?utm_medium=social",
    "https://example.com",
  ),

  multiple_utm_tracker: (
    "https://example.com?utm_medium=social&utm_source=twitter",
    "https://example.com",
  ),

  multiple_utm_tracker_with_others: (
    "https://example.com?utm_medium=social&foo=bar&utm_source=twitter",
    "https://example.com?foo=bar",
  ),

  url_with_spaces: (
    "https://example.com?foo=bar%20baz",
    "https://example.com?foo=bar%20baz"
  ),

  url_with_numerals: (
    "https://example.com?foo=bar0baz",
    "https://example.com?foo=bar0baz"
  ),
}
