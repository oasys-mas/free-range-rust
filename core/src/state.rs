use color_eyre::Result;

pub trait IndexView<'a> {
    type View;

    fn index_view(&'a self, idx: usize) -> Self::View;

    fn index_views(&'a self, indices: &[usize]) -> Vec<Self::View> {
        indices.iter().map(|&i| self.index_view(i)).collect()
    }
}

pub trait State<'a>: IndexView<'a> {
    type Config;

    fn clear(&mut self);

    fn initialize(config: &Self::Config) -> Result<()>;
}
