use crate::error::GreedError;
use std::future::Future;

pub async fn fetch_all<T, F, Fut, Page>(mut fetch_page: F) -> Result<Vec<T>, GreedError>
where
    F: FnMut(Option<Page>) -> Fut,
    Fut: Future<Output = Result<(Vec<T>, Option<Page>), GreedError>>,
{
    let mut current_page: Option<Page> = None;
    let mut all_results = Vec::<T>::new();
    loop {
        let (mut results, next_page) = fetch_page(current_page).await?;
        all_results.append(&mut results);
        current_page = next_page;
        if current_page.is_none() {
            break;
        }
    }
    Ok(all_results)
}

#[cfg(test)]
mod tests {
    use crate::error::GreedError;
    use crate::pager;

    #[tokio::test]
    async fn fetch_all_fails() {
        pager::fetch_all::<(), _, _, ()>(|_| async { Err(GreedError::new("failed")) })
            .await
            .expect_err("expected an error");
    }

    #[tokio::test]
    async fn fetch_all_single_page() {
        let results = pager::fetch_all::<_, _, _, ()>(|_| async { Ok((vec![1, 2], None)) })
            .await
            .expect("failed to fetch results");
        assert_eq!(results, vec![1, 2])
    }

    #[tokio::test]
    async fn fetch_all_multiple_pages() {
        let results = pager::fetch_all(|page| async {
            let result = test_fetch(page).await;
            let next_page = result.first().map(|v| v.to_string());
            Ok((result, next_page))
        })
        .await
        .expect("failed to fetch results");
        assert_eq!(results, vec![0, 1, 2, 3])
    }

    async fn test_fetch(page: Option<String>) -> Vec<u32> {
        match page {
            None => vec![0],
            Some(token) => match token.as_str() {
                "0" => vec![1],
                "1" => vec![2, 3],
                _ => vec![],
            },
        }
    }
}
