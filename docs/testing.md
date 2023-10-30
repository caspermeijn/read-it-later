<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
<!-- Copyright 2023 Casper Meijn <casper@meijn.net> -->

# Manual testing
This chapter contains instructions for manually testing Read It Later. Following these steps, you will have seen the different views of the application and executed all actions. This document is more a guideline than a strict instruction. Feel free to deviate from the steps.

## Server
You could start a test server using the instructions at https://github.com/caspermeijn/wallabag-test-server

This server's `wallabag` account already contains a set of articles. When you restart this server, all changes are lost. So you can experiment as much as you like.

## Login
During the first startup, the application opens the login page. The page welcomes the user and requests login information.

With the wrong credentials, the login button will inform the user that logging in failed.

With the correct credentials, the articles are downloaded and shown in the article list.

## Article lists
After login, the article list is opened. The list has three different views:
- `Unread` contains the articles not marked as archived and not marked as favorite.
- `Favorites` contains all articles marked favorite (regardless of archive status).
- `Archive` contains the articles marked as archived.

When an article is opened and the archive or favorite status is changed, the lists are updated.

Each article shows the title, article information, summary, and an optional picture.

When a list is empty, a placeholder is shown with the text `Pretty clean!`.

## Add a new article
When the article list is visible, a menu option is available to `Add Article...`. Choosing this option will replace the menu bar with a URL input. You can leave this using the back button or enter a URL and press `Save`. The article is downloaded and added to the `Unread` article list.

Example URL:
- https://blog.gtk.org/2023/02/09/updates-from-inside-gtk/
- https://gtk-rs.org/blog/2023/02/10/new-release.html

## Article viewer
When an article from the list is clicked, the full article is opened. It starts with the title and article information. Then the article's body is shown.

In the title bar, the archive and favorite options are directly available. Clicking them will show a progress bar while the status is sent to the server. The icon will indicate the current status of the option.

A menu option is available to `Open website...`. Choosing this option will open the web browser to the URL of the article.

A menu option is available to `Delete article`. Choosing this option will remove the article from the server and the application.

Right-click the article, most controls of the underlying webview are disabled. For example, you should not be able to navigate back and forwards.

Click a link, the article webview doesn't navigate and the link is opened a browser.

## Preferences
When the article list is visible, a menu option is available to `Preferences`. Choosing this option will open the preferences window. Some account information is shown in this window.

## Shortcuts
When the article list is visible, a menu option is available to `Keyboard Shortcuts`. Choosing this option will open the shortcuts window. All shortcuts available in the application are shown here.

## About
When the article list is visible, a menu option is available to `About Read It Later`. Choosing this option will open the about window. The logo, application name, and version are shown. Links are available to the website, credits, and legal.

## Logout
When the article list is visible, a menu option is available to `Logout`. Choosing this option will disconnect from the server and open the login page.

## Responsive design
All views should be useable on large monitors (fullscreen 1080p) and on small screens (phone screens). Test by resizing the window from small to big. The contents should resize dynamically to fit the window.

## Translation
All visible text except the article contents should be translated into your native language.

## Application restart
When logged in and the application is restarted, it will open on the unread article list.
