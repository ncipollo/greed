use std::future::Future;

pub async fn fetch_all<T, F, Fut, Page>(mut fetch_page: F) -> Vec<T>
where
    F: FnMut(Option<Page>) -> Fut,
    Fut: Future<Output = (Vec<T>, Option<Page>)>,
{
    let mut current_page: Option<Page> = None;
    let mut all_results = Vec::<T>::new();
    loop {
        let (mut results, next_page) = fetch_page(current_page).await;
        all_results.append(&mut results);
        current_page = next_page;
        if current_page.is_none() {
            break;
        }
    }
    all_results
}
