pub(crate) fn rule(
    build_ctx: &crate::builder::BuildCtx,
    node: &rnix::SyntaxNode,
) -> std::collections::LinkedList<crate::builder::Step> {
    let mut steps = std::collections::LinkedList::new();

    let mut children: Vec<crate::children2::Child> =
        crate::children2::new(build_ctx, node).collect();

    let vertical = build_ctx.vertical
        || children
            .iter()
            .any(|child| child.has_inline_comment || child.has_trivialities);

    let last = children.pop().unwrap();
    let mut children = children.into_iter().peekable();

    // inherit
    let child = children.next().unwrap();
    steps.push_back(crate::builder::Step::Format(child.element));
    if vertical {
        steps.push_back(crate::builder::Step::Indent);
    }

    if let Some(text) = child.inline_comment {
        steps.push_back(crate::builder::Step::Whitespace);
        steps.push_back(crate::builder::Step::Comment(text));
        steps.push_back(crate::builder::Step::NewLine);
        steps.push_back(crate::builder::Step::Pad);
    } else if vertical {
        steps.push_back(crate::builder::Step::NewLine);
        steps.push_back(crate::builder::Step::Pad);
    }

    for trivia in child.trivialities {
        match trivia {
            crate::children2::Trivia::Comment(text) => {
                steps.push_back(crate::builder::Step::Comment(text));
                steps.push_back(crate::builder::Step::NewLine);
                steps.push_back(crate::builder::Step::Pad);
            }
            crate::children2::Trivia::Newlines(_) => {}
        }
    }

    let mut any_comments = false;
    if matches!(children.peek().unwrap().element.kind(), rnix::SyntaxKind::NODE_INHERIT_FROM) {
        let child = children.next().unwrap();
        any_comments = any_comments || child.has_inline_comment || child.has_comments;
        format_inherit_child(&mut steps, vertical, child, true);
    }

    let mut ident_children = children.collect::<Vec<_>>();
    any_comments = any_comments || ident_children.iter().any(|child| child.has_inline_comment || child.has_comments);
    if !any_comments {
        ident_children.sort_unstable_by(|a, b| human_sort::compare(&child_text(a), &child_text(b)))
    }
    for child in ident_children.into_iter() {
        format_inherit_child(&mut steps, vertical, child, true);
    }

    format_inherit_child(&mut steps, vertical, last, false);

    if vertical {
        steps.push_back(crate::builder::Step::Dedent);
    }

    steps
}

fn child_text(child: &crate::children2::Child) -> String {
    match &child.element {
        rnix::SyntaxElement::Node(node) => node.text().to_string(),
        rnix::SyntaxElement::Token(token) => token.text().to_string()
    }
}

fn format_inherit_child(
    steps: &mut std::collections::LinkedList<crate::builder::Step>,
    vertical: bool,
    child: crate::children2::Child,
    not_last_child: bool,
) {
    if vertical {
        steps.push_back(crate::builder::Step::FormatWider(child.element));

        if let Some(text) = child.inline_comment {
            steps.push_back(crate::builder::Step::Whitespace);
            steps.push_back(crate::builder::Step::Comment(text));
            steps.push_back(crate::builder::Step::NewLine);
            // Only add padding if there are no `trivialities` (that is,
            // there's no extra `Newlines(_)` to be added)
            // or if the first one is a comment (that is, it'll need
            // to be indented to match the content).
            if matches!(
                child.trivialities.front(),
                None | Some(crate::children2::Trivia::Comment(_))
            ) {
                steps.push_back(crate::builder::Step::Pad);
            }
        } else if (not_last_child && !child.has_trivialities)
            || matches!(
                child.trivialities.front(),
                Some(crate::children2::Trivia::Comment(_))
            )
        {
            steps.push_back(crate::builder::Step::NewLine);
            steps.push_back(crate::builder::Step::Pad);
        }

        let mut trivia_iter = child.trivialities.into_iter().peekable();
        while let Some(trivia) = trivia_iter.next() {
            match trivia {
                crate::children2::Trivia::Comment(text) => {
                    steps.push_back(crate::builder::Step::Comment(text));
                    // If the next `trivia` is a newline, don't add newlines
                    // and padding at the
                    // end of this iteration, as it will lead to a new blank
                    // line in the output.
                    if matches!(
                        trivia_iter.peek(),
                        Some(crate::children2::Trivia::Newlines(_))
                    ) {
                        continue;
                    }
                }
                crate::children2::Trivia::Newlines(_) => {}
            }
            if not_last_child {
                steps.push_back(crate::builder::Step::NewLine);
                steps.push_back(crate::builder::Step::Pad);
            }
        }
    } else {
        if not_last_child {
            steps.push_back(crate::builder::Step::Whitespace);
        }
        steps.push_back(crate::builder::Step::Format(child.element));
    }
}
