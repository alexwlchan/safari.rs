use reqwest::Client;

use urlencoding::{encode as urlencode};
use urlparse::{Query, parse_qs, urlparse, urlunparse, Url};


/// Follow redirects to resolve the final location of a URL
pub fn resolve(url: &str) -> String {
  let client = Client::new();
  client.head(url)
        .send()
        .unwrap()
        .url()
        .as_str()
        .to_owned()
}


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

  // Always get the non-mobile version of nytimes.com URLs
  if parsed_url.netloc == "mobile.nytimes.com" {
    parsed_url.netloc = String::from("nytimes.com");
  }

  // Remove any tracking junk from Amazon URLs so they're not a
  // ridiculous length
  if (parsed_url.netloc == "www.amazon.co.uk") ||
     (parsed_url.netloc == "smile.amazon.co.uk"){
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

  // Strip tracking junk from TikTok URLs
  if parsed_url.netloc == "www.tiktok.com" {
    parsed_url.query = None;
  }

  // Remove '#notes' from Tumblr URLs
  if parsed_url.netloc.ends_with("tumblr.com") {
    parsed_url.fragment = match parsed_url.fragment {
      Some(fragment) => if fragment == "notes" { None } else { Some(fragment) },
      None => None,
    };
  }

  // Remove &feature=youtu.be from YouTube URLs
  if parsed_url.netloc.ends_with("youtube.com") {
    remove_query_param(&mut parsed_url, "feature");
    remove_query_param(&mut parsed_url, "app");
  }

  // Remove any UTM tracking parameters and Cloudflare parameters from all URLs
  parsed_url.query = match parsed_url.query {
    Some(qs) => {
      let mut query = parse_qs(&qs);
      let utm_keys: Vec<_> = query
        .keys()
        .filter(|key| key.starts_with("utm_") || key.starts_with("__cf"))
        .map(|k| k.clone())
        .collect();
      for key in utm_keys {
        query.remove(&key);
      }
      encode_querystring(query)
    },
    None => None
  };

  // Tidy up the query and anchor links in modules on docs.python.org
  if parsed_url.netloc == "docs.python.org" {

    // If this is a module page, scrap any module- fragment.
    parsed_url.fragment = match parsed_url.fragment {
      Some(fragment) => if fragment.starts_with("module-") { None } else { Some(fragment) },
      None => None,
    };

    // Scrap the highlight
    remove_query_param(&mut parsed_url, "highlight");
  }

  // If I'm on a GitHub pull request and looking at the files tab,
  // link to the top of the pull request.
  if parsed_url.netloc == "github.com" {
    if parsed_url.path.ends_with("/files") {
      parsed_url.path = parsed_url.path.replace("/files", "");
    }
  }

  // Remove tracking query parameters from telegraph.co.uk URLs
  // https://github.com/alexwlchan/safari.rs/issues/48
  if parsed_url.netloc == "www.telegraph.co.uk" {
    remove_query_param(&mut parsed_url, "WT.mc_id");
  }

  // Remove Google Analytics parameters from Etsy URLs.
  if parsed_url.netloc == "www.etsy.com" {
    remove_query_param(&mut parsed_url, "awc");
    remove_query_param(&mut parsed_url, "frs");
    remove_query_param(&mut parsed_url, "source");
    remove_query_param(&mut parsed_url, "pro");

    parsed_url.query = match parsed_url.query {
      Some(qs) => {
        let mut query = parse_qs(&qs);
        let utm_keys: Vec<_> = query
          .keys()
          .filter(|key|
            key.starts_with("ga_") ||
            key.starts_with("ref") ||
            key.starts_with("organic_search_click")
          )
          .map(|k| k.clone())
          .collect();
        for key in utm_keys {
          query.remove(&key);
        }
        encode_querystring(query)
      },
      None => None
    };
  }

  // Un-mobile-ify blogspot links.
  if parsed_url.netloc.ends_with("blogspot.com") {
    remove_query_param(&mut parsed_url, "m");
  }

  // Un-mobile-ify blogspot links.
  if parsed_url.netloc == "www.redbubble.com" {
    remove_query_param(&mut parsed_url, "ref");
    remove_query_param(&mut parsed_url, "asc");
  }

  // Always remove the _ga Google Analytics tracking parameter.
  remove_query_param(&mut parsed_url, "_ga");

  // Remove tracking parameters from stacks.wellcomecollection.org URLs,
  // which are from Medium.
  if parsed_url.netloc == "stacks.wellcomecollection.org" {
    remove_query_param(&mut parsed_url, "source");
  }

  // Remove tracking parameters from Wordery URLs.
  if parsed_url.netloc == "wordery.com" {
    remove_query_param(&mut parsed_url, "cTrk");
  }

  // Remove the share parameter from Twitter URLs.
  if parsed_url.netloc == "twitter.com" {
    remove_query_param(&mut parsed_url, "s");
  }

  // Convert links to questions on Stack Overflow to insert my sharing
  // link for the Announcer badge.
  // This code is autogenerated by the `se_referral_autogen.py` script.
  fix_se_referral(&mut parsed_url, "academia.stackexchange.com", "7658");
  fix_se_referral(&mut parsed_url, "anime.stackexchange.com", "7674");
  fix_se_referral(&mut parsed_url, "apple.stackexchange.com", "14295");
  fix_se_referral(&mut parsed_url, "area51.stackexchange.com", "47881");
  fix_se_referral(&mut parsed_url, "askubuntu.com", "265738");
  fix_se_referral(&mut parsed_url, "aviation.stackexchange.com", "1372");
  fix_se_referral(&mut parsed_url, "bicycles.stackexchange.com", "17362");
  fix_se_referral(&mut parsed_url, "biology.stackexchange.com", "16115");
  fix_se_referral(&mut parsed_url, "bricks.stackexchange.com", "445");
  fix_se_referral(&mut parsed_url, "chemistry.stackexchange.com", "5443");
  fix_se_referral(&mut parsed_url, "chess.stackexchange.com", "2564");
  fix_se_referral(&mut parsed_url, "christianity.stackexchange.com", "10196");
  fix_se_referral(&mut parsed_url, "codegolf.stackexchange.com", "13285");
  fix_se_referral(&mut parsed_url, "codereview.stackexchange.com", "36525");
  fix_se_referral(&mut parsed_url, "cogsci.stackexchange.com", "7973");
  fix_se_referral(&mut parsed_url, "communitybuilding.stackexchange.com", "372");
  fix_se_referral(&mut parsed_url, "cooking.stackexchange.com", "25134");
  fix_se_referral(&mut parsed_url, "crypto.stackexchange.com", "1185");
  fix_se_referral(&mut parsed_url, "diy.stackexchange.com", "25263");
  fix_se_referral(&mut parsed_url, "dsp.stackexchange.com", "8360");
  fix_se_referral(&mut parsed_url, "earthscience.stackexchange.com", "352");
  fix_se_referral(&mut parsed_url, "elementaryos.stackexchange.com", "3586");
  fix_se_referral(&mut parsed_url, "english.stackexchange.com", "22597");
  fix_se_referral(&mut parsed_url, "gaming.stackexchange.com", "73524");
  fix_se_referral(&mut parsed_url, "gardening.stackexchange.com", "813");
  fix_se_referral(&mut parsed_url, "gis.stackexchange.com", "26054");
  fix_se_referral(&mut parsed_url, "graphicdesign.stackexchange.com", "19347");
  fix_se_referral(&mut parsed_url, "law.stackexchange.com", "2488");
  fix_se_referral(&mut parsed_url, "lifehacks.stackexchange.com", "11245");
  fix_se_referral(&mut parsed_url, "linguistics.stackexchange.com", "2722");
  fix_se_referral(&mut parsed_url, "math.stackexchange.com", "24160");
  fix_se_referral(&mut parsed_url, "matheducators.stackexchange.com", "661");
  fix_se_referral(&mut parsed_url, "mathematica.stackexchange.com", "5190");
  fix_se_referral(&mut parsed_url, "mathoverflow.net", "38734");
  fix_se_referral(&mut parsed_url, "mechanics.stackexchange.com", "14629");
  fix_se_referral(&mut parsed_url, "meta.stackexchange.com", "226928");
  fix_se_referral(&mut parsed_url, "money.stackexchange.com", "13518");
  fix_se_referral(&mut parsed_url, "movies.stackexchange.com", "9285");
  fix_se_referral(&mut parsed_url, "networkengineering.stackexchange.com", "16668");
  fix_se_referral(&mut parsed_url, "opensource.stackexchange.com", "2909");
  fix_se_referral(&mut parsed_url, "parenting.stackexchange.com", "14304");
  fix_se_referral(&mut parsed_url, "patents.stackexchange.com", "3882");
  fix_se_referral(&mut parsed_url, "philosophy.stackexchange.com", "16838");
  fix_se_referral(&mut parsed_url, "photo.stackexchange.com", "29700");
  fix_se_referral(&mut parsed_url, "physics.stackexchange.com", "38614");
  fix_se_referral(&mut parsed_url, "politics.stackexchange.com", "5590");
  fix_se_referral(&mut parsed_url, "productivity.stackexchange.com", "1990");
  fix_se_referral(&mut parsed_url, "puzzling.stackexchange.com", "4692");
  fix_se_referral(&mut parsed_url, "rpg.stackexchange.com", "22823");
  fix_se_referral(&mut parsed_url, "salesforce.stackexchange.com", "36215");
  fix_se_referral(&mut parsed_url, "scifi.stackexchange.com", "3567");
  fix_se_referral(&mut parsed_url, "security.stackexchange.com", "9814");
  fix_se_referral(&mut parsed_url, "serverfault.com", "206273");
  fix_se_referral(&mut parsed_url, "skeptics.stackexchange.com", "5416");
  fix_se_referral(&mut parsed_url, "softwareengineering.stackexchange.com", "94977");
  fix_se_referral(&mut parsed_url, "space.stackexchange.com", "1003");
  fix_se_referral(&mut parsed_url, "sqa.stackexchange.com", "7301");
  fix_se_referral(&mut parsed_url, "stackapps.com", "21515");
  fix_se_referral(&mut parsed_url, "stackoverflow.com", "1558022");
  fix_se_referral(&mut parsed_url, "stats.stackexchange.com", "32450");
  fix_se_referral(&mut parsed_url, "superuser.com", "243137");
  fix_se_referral(&mut parsed_url, "tex.stackexchange.com", "9668");
  fix_se_referral(&mut parsed_url, "travel.stackexchange.com", "12415");
  fix_se_referral(&mut parsed_url, "unix.stackexchange.com", "43183");
  fix_se_referral(&mut parsed_url, "ux.stackexchange.com", "9976");
  fix_se_referral(&mut parsed_url, "webapps.stackexchange.com", "45296");
  fix_se_referral(&mut parsed_url, "webmasters.stackexchange.com", "35749");
  fix_se_referral(&mut parsed_url, "workplace.stackexchange.com", "14106");
  fix_se_referral(&mut parsed_url, "worldbuilding.stackexchange.com", "2575");
  fix_se_referral(&mut parsed_url, "writers.stackexchange.com", "11018");

  urlunparse(parsed_url)
}


/// Remove a query parameter from a URL.
///
/// - `parsed_url` - the `Url` structure returned by urlparse
/// - `query_param` - name of the query parameter to remove.
///
fn remove_query_param(parsed_url: &mut Url, query_param: &str) {
  parsed_url.query = match parsed_url.query {
    Some(ref qs) => {
      let mut query = parse_qs(&qs);
      query.remove(query_param);
      encode_querystring(query)
    },
    None => None
  };
}


/// Turn a URL into a Stack Exchange referral link.
///
/// - `parsed_url` - the `Url` structure returned by urlparse
/// - `hostname` - hostname of the SE site in question
/// - `user_id` - your user ID on the SE site
///
fn fix_se_referral(parsed_url: &mut Url, hostname: &str, user_id: &str) {

  if parsed_url.netloc != hostname {
    return;
  }

  // A question URL is of the form
  //
  //    http://stackoverflow.com/questions/:question_id/:question_title
  //
  if !parsed_url.path.starts_with("/questions") {
    return;
  }

  // Take a copy of the original path to get around ownership rules
  let original_path = parsed_url.path.to_owned();

  let new_path = match original_path.split("/").nth(2) {
    Some(path_component) => {
      // Check it's a number
      match path_component.parse::<i32>() {
        Ok(q_id) => {
          // Check if there's an answer fragment
          match parsed_url.fragment.to_owned() {
            Some(fragment) => match fragment.parse::<i32>() {
              Ok(ans_id) => Some(format!("/a/{}/{}", ans_id, user_id)),
              Err(_) => None,
            },
            None => Some(format!("/q/{}/{}", q_id, user_id)),
          }
        },
        Err(_) => None,
      }
    }
    None => None,
  };

  // If we got something interesting, update the URL.
  match new_path {
    Some(p) => {
      parsed_url.path = p;
      parsed_url.fragment = None;
    },
    None => (),
  };
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

  amazon_smile_product: (
    "https://smile.amazon.co.uk/dp/B01DFKBL68/ref=gw_aucc_comb_AB_clean_2?pf_rd_r=N1MF2PWADRHS55427ETG&pf_rd_p=c11a2c11-c670-46ff-87a1-c1eef4fad652",
    "https://smile.amazon.co.uk/dp/B01DFKBL68/"
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

  youtube_with_app: (
    "https://www.youtube.com/watch?app=desktop&v=51HbzsFhh04",
    "https://www.youtube.com/watch?v=51HbzsFhh04"
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

  python_docs_bare: (
    "https://docs.python.org/3.5/library/subprocess.html",
    "https://docs.python.org/3.5/library/subprocess.html"
  ),

  python_docs_with_anchor: (
    "https://docs.python.org/3.5/library/subprocess.html#subprocess.run",
    "https://docs.python.org/3.5/library/subprocess.html#subprocess.run"
  ),

  python_docs_with_highlight: (
    "https://docs.python.org/3.5/library/subprocess.html?highlight=subprocess",
    "https://docs.python.org/3.5/library/subprocess.html"
  ),

  python_docs_with_highlight_anchor: (
    "https://docs.python.org/3.5/library/subprocess.html?highlight=subprocess#subprocess.run",
    "https://docs.python.org/3.5/library/subprocess.html#subprocess.run"
  ),

  python_docs_with_highlight_module_anchor: (
    "https://docs.python.org/3.5/library/subprocess.html?highlight=subprocess#module-subprocess",
    "https://docs.python.org/3.5/library/subprocess.html"
  ),

  stack_overflow_non_question: (
    "http://stackoverflow.com/questions/tagged/html+regex",
    "http://stackoverflow.com/questions/tagged/html+regex"
  ),

  stack_overflow_question: (
    "http://stackoverflow.com/questions/1732348/regex-match-open-tags-except-xhtml-self-contained-tags",
    "http://stackoverflow.com/q/1732348/1558022"
  ),

  stack_overflow_answer: (
    "http://stackoverflow.com/questions/82831/how-do-i-check-whether-a-file-exists-using-python#82852",
    "http://stackoverflow.com/a/82852/1558022"
  ),

  sff_se_non_question: (
    "https://scifi.stackexchange.com/review",
    "https://scifi.stackexchange.com/review"
  ),

  sff_se_question: (
    "https://scifi.stackexchange.com/questions/58980/how-did-lupin-forget-there-was-a-full-moon",
    "https://scifi.stackexchange.com/q/58980/3567"
  ),

  sff_se_answer: (
    "https://scifi.stackexchange.com/questions/39201/which-owls-did-fred-and-george-weasley-achieve/39218#39218",
    "https://scifi.stackexchange.com/a/39218/3567"
  ),

  telegraph_bare: (
    "http://www.telegraph.co.uk/news/2017/06/09/ruth-davidson-planning-scottish-tory-breakaway-challenges-theresa/",
    "http://www.telegraph.co.uk/news/2017/06/09/ruth-davidson-planning-scottish-tory-breakaway-challenges-theresa/"
  ),

  strip_telegraph_tracking: (
    "http://www.telegraph.co.uk/news/2017/06/09/ruth-davidson-planning-scottish-tory-breakaway-challenges-theresa/?WT.mc_id=tmg_share_tw",
    "http://www.telegraph.co.uk/news/2017/06/09/ruth-davidson-planning-scottish-tory-breakaway-challenges-theresa/"
  ),

  se_comments: (
    "https://stackoverflow.com/questions/406230/regular-expression-to-match-a-line-that-doesnt-contain-a-word#comment9209422_406230",
    "https://stackoverflow.com/questions/406230/regular-expression-to-match-a-line-that-doesnt-contain-a-word#comment9209422_406230"
  ),

  ga_tracking_parameter: (
    "https://example.org?_ga=1234",
    "https://example.org"
  ),

  ga_tracking_parameter_and_others: (
    "https://example.org?_ga=1234&foo=bar",
    "https://example.org?foo=bar"
  ),

  nytimes_mobile_url: (
    "https://mobile.nytimes.com/2017/02/24/style/modern-love-when-your-greatest-romance-is-friendship.html",
    "https://nytimes.com/2017/02/24/style/modern-love-when-your-greatest-romance-is-friendship.html"
  ),

  nytimes_non_mobile_url: (
    "https://nytimes.com/2017/02/24/style/modern-love-when-your-greatest-romance-is-friendship.html",
    "https://nytimes.com/2017/02/24/style/modern-love-when-your-greatest-romance-is-friendship.html"
  ),

  github_pr: (
    "https://github.com/wellcometrust/platform/pull/1892",
    "https://github.com/wellcometrust/platform/pull/1892"
  ),

  github_pr_with_files: (
    "https://github.com/wellcometrust/platform/pull/1892/files",
    "https://github.com/wellcometrust/platform/pull/1892"
  ),

  etsy_link: (
    "https://www.etsy.com/uk/listing/473409127/space-sampler-cross-stitch-pattern-pdf?ga_order=most_relevant&ref=sr_gallery-1-3&ga_view_type=gallery&organic_search_click=1&ga_search_type=all&ga_search_query=space%20sampler",
    "https://www.etsy.com/uk/listing/473409127/space-sampler-cross-stitch-pattern-pdf"
  ),

  etsy_section_link: (
    "https://www.etsy.com/uk/shop/Vitraaze?source=aw&section_id=17807678&awc=6091_1577546787_d9f1b918042bbd02fe406d61dae715e7",
    "https://www.etsy.com/uk/shop/Vitraaze?section_id=17807678"
  ),

  etsy_pro_link: (
    "https://www.etsy.com/uk/listing/704478659/gothic-cathedral-window-ornate-cross?pro=1",
    "https://www.etsy.com/uk/listing/704478659/gothic-cathedral-window-ornate-cross"
  ),

  etsy_link_with_tracking: (
    "https://www.etsy.com/uk/listing/731582894/personalised-song-lyric-or-poetry?frs=1",
    "https://www.etsy.com/uk/listing/731582894/personalised-song-lyric-or-poetry",
  ),

  blogspot: (
    "https://example.blogspot.com/2020/01/01/my_first_post.html",
    "https://example.blogspot.com/2020/01/01/my_first_post.html"
  ),

  blogspot_mobile_link: (
    "https://example.blogspot.com/2020/01/01/my_first_post.html?m=1",
    "https://example.blogspot.com/2020/01/01/my_first_post.html"
  ),

  redbubble_plain: (
    "https://www.redbubble.com/people/m1sfire/works/42386806-pixel-pride-genderfluid",
    "https://www.redbubble.com/people/m1sfire/works/42386806-pixel-pride-genderfluid"
  ),

  redbubble_referrer: (
    "https://www.redbubble.com/people/m1sfire/works/42386806-pixel-pride-genderfluid?ref=recent-owner",
    "https://www.redbubble.com/people/m1sfire/works/42386806-pixel-pride-genderfluid"
  ),

  redbubble_asc: (
    "https://www.redbubble.com/people/m1sfire/works/42386806-pixel-pride-genderfluid?asc=u",
    "https://www.redbubble.com/people/m1sfire/works/42386806-pixel-pride-genderfluid"
  ),

  stacks_medium_referrer: (
    "https://stacks.wellcomecollection.org/a-sprinkling-of-azure-6cef6e150fb2?source=collection_home---4------0———————————",
    "https://stacks.wellcomecollection.org/a-sprinkling-of-azure-6cef6e150fb2"
  ),

  wordery_with_tracking_param: (
    "https://wordery.com/digging-for-words-angela-burke-kunkel-9781984892638?cTrk=MTgwMTkyNTkxfDVmYjgzYzU5YzQ4Mjc6MToxOjVmYjgzYzE4ZTdhNzQ4LjczMzU3NDQxOjVhMzBiMGYx",
    "https://wordery.com/digging-for-words-angela-burke-kunkel-9781984892638"
  ),

  twitter_with_share_link: (
    "https://twitter.com/ScotRail/status/1355928580895731713?s=20",
    "https://twitter.com/ScotRail/status/1355928580895731713"
  ),

  url_with_cloudflare_query_param: (
    "https://example.org/page?__cf_chl_jschl_tk__=1",
    "https://example.org/page"
  ),

  tiktok_with_tracking_junk: (
  "https://www.tiktok.com/@example/video/1234567890?sec_user_id=ABCDEFGHIJ&u_code=dgfffl6mjl6be3&share_app_id=1233&timestamp=1631110682",
    "https://www.tiktok.com/@example/video/1234567890"
  ),
}
