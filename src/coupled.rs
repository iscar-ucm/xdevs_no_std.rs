use crate::component::Component;
use crate::port::UnsafePort;

pub unsafe trait UnsafeCoupled {
    type Components: Components;

    type Input: UnsafePort;

    type Output: UnsafePort;

    fn divide(&self) -> (&Self::Components, &Component<Self::Input, Self::Output>);

    fn divide_mut(
        &mut self,
    ) -> (
        &mut Self::Components,
        &mut Component<Self::Input, Self::Output>,
    );

    #[inline]
    fn get_components(&self) -> &Self::Components {
        let (components, _) = self.divide();
        components
    }

    #[inline]
    fn get_components_mut(&mut self) -> &mut Self::Components {
        let (components, _) = self.divide_mut();
        components
    }

    #[inline]
    fn get_component(&self) -> &Component<Self::Input, Self::Output> {
        let (_, component) = self.divide();
        component
    }

    #[inline]
    fn get_component_mut(&mut self) -> &mut Component<Self::Input, Self::Output> {
        let (_, component) = self.divide_mut();
        component
    }
}

pub trait Components {
    fn start(&mut self) -> f64;
    fn stop(&mut self);
    fn lambda(&mut self);
    fn delta(&mut self) -> f64;
}
