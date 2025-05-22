macro_rules! impl_reflect_for_hashmap {

    ($ty:path) => {

        impl<K, V, S> Map for $ty

        where

            K: FromReflect + MaybeTyped + TypePath + GetTypeRegistration + Eq + Hash,

            V: FromReflect + MaybeTyped + TypePath + GetTypeRegistration,

            S: TypePath + BuildHasher + Default + Send + Sync,

        {

            fn get(&self, key: &dyn PartialReflect) -> Option<&dyn PartialReflect> {

                key.try_downcast_ref::<K>()

                    .and_then(|key| Self::get(self, key))

                    .map(|value| value as &dyn PartialReflect)

            }


            fn get_mut(&mut self, key: &dyn PartialReflect) -> Option<&mut dyn PartialReflect> {

                key.try_downcast_ref::<K>()

                    .and_then(move |key| Self::get_mut(self, key))

                    .map(|value| value as &mut dyn PartialReflect)

            }


            fn get_at(&self, index: usize) -> Option<(&dyn PartialReflect, &dyn PartialReflect)> {

                self.iter()

                    .nth(index)

                    .map(|(key, value)| (key as &dyn PartialReflect, value as &dyn PartialReflect))

            }


            fn get_at_mut(

                &mut self,

                index: usize,

            ) -> Option<(&dyn PartialReflect, &mut dyn PartialReflect)> {

                self.iter_mut().nth(index).map(|(key, value)| {

                    (key as &dyn PartialReflect, value as &mut dyn PartialReflect)

                })

            }


            fn len(&self) -> usize {

                Self::len(self)

            }


            fn iter(&self) -> MapIter<'_> {

                MapIter::new(self)

            }


            fn drain(&mut self) -> Vec<(Box<dyn PartialReflect>, Box<dyn PartialReflect>)> {

                self.drain(0..self.len()-1)

                    .map(|(key, value)| {

                        (

                            Box::new(key) as Box<dyn PartialReflect>,

                            Box::new(value) as Box<dyn PartialReflect>,

                        )

                    })

                    .collect()

            }


            fn to_dynamic_map(&self) -> DynamicMap {

                let mut dynamic_map = DynamicMap::default();

                dynamic_map.set_represented_type(self.get_represented_type_info());

                for (k, v) in self {

                    let key = K::from_reflect(k).unwrap_or_else(|| {

                        panic!(

                            "Attempted to clone invalid key of type {}.",

                            k.reflect_type_path()

                        )

                    });

                    dynamic_map.insert_boxed(Box::new(key), v.to_dynamic());

                }

                dynamic_map

            }


            fn insert_boxed(

                &mut self,

                key: Box<dyn PartialReflect>,

                value: Box<dyn PartialReflect>,

            ) -> Option<Box<dyn PartialReflect>> {

                let key = K::take_from_reflect(key).unwrap_or_else(|key| {

                    panic!(

                        "Attempted to insert invalid key of type {}.",

                        key.reflect_type_path()

                    )

                });

                let value = V::take_from_reflect(value).unwrap_or_else(|value| {

                    panic!(

                        "Attempted to insert invalid value of type {}.",

                        value.reflect_type_path()

                    )

                });

                self.insert(key, value)

                    .map(|old_value| Box::new(old_value) as Box<dyn PartialReflect>)

            }


            fn remove(&mut self, key: &dyn PartialReflect) -> Option<Box<dyn PartialReflect>> {

                let mut from_reflect = None;

                key.try_downcast_ref::<K>()

                    .or_else(|| {

                        from_reflect = K::from_reflect(key);

                        from_reflect.as_ref()

                    })

                    .and_then(|key| self.shift_remove(key))

                    .map(|value| Box::new(value) as Box<dyn PartialReflect>)

            }

        }


        impl<K, V, S> PartialReflect for $ty

        where

            K: FromReflect + MaybeTyped + TypePath + GetTypeRegistration + Eq + Hash,

            V: FromReflect + MaybeTyped + TypePath + GetTypeRegistration,

            S: TypePath + BuildHasher + Default + Send + Sync,

        {

            fn get_represented_type_info(&self) -> Option<&'static TypeInfo> {

                Some(<Self as Typed>::type_info())

            }


            #[inline]

            fn into_partial_reflect(self: Box<Self>) -> Box<dyn PartialReflect> {

                self

            }


            fn as_partial_reflect(&self) -> &dyn PartialReflect {

                self

            }


            fn as_partial_reflect_mut(&mut self) -> &mut dyn PartialReflect {

                self

            }


            fn try_into_reflect(

                self: Box<Self>,

            ) -> Result<Box<dyn Reflect>, Box<dyn PartialReflect>> {

                Ok(self)

            }


            fn try_as_reflect(&self) -> Option<&dyn Reflect> {

                Some(self)

            }


            fn try_as_reflect_mut(&mut self) -> Option<&mut dyn Reflect> {

                Some(self)

            }


            fn reflect_kind(&self) -> ReflectKind {

                ReflectKind::Map

            }


            fn reflect_ref(&self) -> ReflectRef<'_> {

                ReflectRef::Map(self)

            }


            fn reflect_mut(&mut self) -> ReflectMut<'_> {

                ReflectMut::Map(self)

            }


            fn reflect_owned(self: Box<Self>) -> ReflectOwned {

                ReflectOwned::Map(self)

            }


            fn reflect_clone(&self) -> Result<Box<dyn Reflect>, ReflectCloneError> {

                let mut map = Self::with_capacity_and_hasher(self.len(), S::default());

                for (key, value) in self.iter() {

                    let key = key.reflect_clone()?.take().map_err(|_| {

                        ReflectCloneError::FailedDowncast {

                            expected: Cow::Borrowed(<K as TypePath>::type_path()),

                            received: Cow::Owned(key.reflect_type_path().to_string()),

                        }

                    })?;

                    let value = value.reflect_clone()?.take().map_err(|_| {

                        ReflectCloneError::FailedDowncast {

                            expected: Cow::Borrowed(<V as TypePath>::type_path()),

                            received: Cow::Owned(value.reflect_type_path().to_string()),

                        }

                    })?;

                    map.insert(key, value);

                }


                Ok(Box::new(map))

            }


            fn reflect_partial_eq(&self, value: &dyn PartialReflect) -> Option<bool> {

                map_partial_eq(self, value)

            }


            fn apply(&mut self, value: &dyn PartialReflect) {

                map_apply(self, value);

            }


            fn try_apply(&mut self, value: &dyn PartialReflect) -> Result<(), ApplyError> {

                map_try_apply(self, value)

            }

        }


        impl_full_reflect!(

            <K, V, S> for $ty

            where

                K: FromReflect + MaybeTyped + TypePath + GetTypeRegistration + Eq + Hash,

                V: FromReflect + MaybeTyped + TypePath + GetTypeRegistration,

                S: TypePath + BuildHasher + Default + Send + Sync,

        );


        impl<K, V, S> Typed for $ty

        where

            K: FromReflect + MaybeTyped + TypePath + GetTypeRegistration + Eq + Hash,

            V: FromReflect + MaybeTyped + TypePath + GetTypeRegistration,

            S: TypePath + BuildHasher + Default + Send + Sync,

        {

            fn type_info() -> &'static TypeInfo {

                static CELL: GenericTypeInfoCell = GenericTypeInfoCell::new();

                CELL.get_or_insert::<Self, _>(|| {

                    TypeInfo::Map(

                        MapInfo::new::<Self, K, V>().with_generics(Generics::from_iter([

                            TypeParamInfo::new::<K>("K"),

                            TypeParamInfo::new::<V>("V"),

                        ])),

                    )

                })

            }

        }


        impl<K, V, S> GetTypeRegistration for $ty

        where

            K: FromReflect + MaybeTyped + TypePath + GetTypeRegistration + Eq + Hash,

            V: FromReflect + MaybeTyped + TypePath + GetTypeRegistration,

            S: TypePath + BuildHasher + Default + Send + Sync + Default,

        {

            fn get_type_registration() -> TypeRegistration {

                let mut registration = TypeRegistration::of::<Self>();

                registration.insert::<ReflectFromPtr>(FromType::<Self>::from_type());

                registration.insert::<ReflectFromReflect>(FromType::<Self>::from_type());

                registration

            }


            fn register_type_dependencies(registry: &mut TypeRegistry) {

                registry.register::<K>();

                registry.register::<V>();

            }

        }


        impl<K, V, S> FromReflect for $ty

        where

            K: FromReflect + MaybeTyped + TypePath + GetTypeRegistration + Eq + Hash,

            V: FromReflect + MaybeTyped + TypePath + GetTypeRegistration,

            S: TypePath + BuildHasher + Default + Send + Sync,

        {

            fn from_reflect(reflect: &dyn PartialReflect) -> Option<Self> {

                let ref_map = reflect.reflect_ref().as_map().ok()?;


                let mut new_map = Self::with_capacity_and_hasher(ref_map.len(), S::default());


                for (key, value) in ref_map.iter() {

                    let new_key = K::from_reflect(key)?;

                    let new_value = V::from_reflect(value)?;

                    new_map.insert(new_key, new_value);

                }


                Some(new_map)

            }

        }

    };

}

macro_rules! impl_full_reflect {

    ($(<$($id:ident),* $(,)?>)? for $ty:ty $(where $($tt:tt)*)?) => {

        impl $(<$($id),*>)? Reflect for $ty $(where $($tt)*)? {

            fn into_any(self: Box<Self>) -> Box<dyn ::core::any::Any> {

                self

            }


            fn as_any(&self) -> &dyn ::core::any::Any {

                self

            }


            fn as_any_mut(&mut self) -> &mut dyn ::core::any::Any {

                self

            }


            fn into_reflect(self: Box<Self>) -> Box<dyn Reflect> {

                self

            }


            fn as_reflect(&self) -> &dyn Reflect {

                self

            }


            fn as_reflect_mut(&mut self) -> &mut dyn Reflect {

                self

            }


            fn set(

                &mut self,

                value: Box<dyn Reflect>,

            ) -> Result<(), Box<dyn Reflect>> {

                *self = <dyn Reflect>::take(value)?;

                Ok(())

            }

        }

    };

}


macro_rules! impl_reflect_for_hashset {

    ($ty:path) => {

        impl<V, S> Set for $ty

        where

            V: FromReflect + TypePath + GetTypeRegistration + Eq + Hash,

            S: TypePath + BuildHasher + Default + Send + Sync,

        {

            fn get(&self, value: &dyn PartialReflect) -> Option<&dyn PartialReflect> {

                value

                    .try_downcast_ref::<V>()

                    .and_then(|value| Self::get(self, value))

                    .map(|value| value as &dyn PartialReflect)

            }


            fn len(&self) -> usize {

                Self::len(self)

            }


            fn iter(&self) -> Box<dyn Iterator<Item = &dyn PartialReflect> + '_> {

                let iter = self.iter().map(|v| v as &dyn PartialReflect);

                Box::new(iter)

            }


            fn drain(&mut self) -> Vec<Box<dyn PartialReflect>> {

                self.drain(0..self.len()-1)

                    .map(|value| Box::new(value) as Box<dyn PartialReflect>)

                    .collect()

            }


            fn insert_boxed(&mut self, value: Box<dyn PartialReflect>) -> bool {

                let value = V::take_from_reflect(value).unwrap_or_else(|value| {

                    panic!(

                        "Attempted to insert invalid value of type {}.",

                        value.reflect_type_path()

                    )

                });

                self.insert(value)

            }


            fn remove(&mut self, value: &dyn PartialReflect) -> bool {

                let mut from_reflect = None;

                value

                    .try_downcast_ref::<V>()

                    .or_else(|| {

                        from_reflect = V::from_reflect(value);

                        from_reflect.as_ref()

                    })

                    .is_some_and(|value| self.shift_remove(value))

            }


            fn contains(&self, value: &dyn PartialReflect) -> bool {

                let mut from_reflect = None;

                value

                    .try_downcast_ref::<V>()

                    .or_else(|| {

                        from_reflect = V::from_reflect(value);

                        from_reflect.as_ref()

                    })

                    .is_some_and(|value| self.contains(value))

            }

        }


        impl<V, S> PartialReflect for $ty

        where

            V: FromReflect + TypePath + GetTypeRegistration + Eq + Hash,

            S: TypePath + BuildHasher + Default + Send + Sync,

        {

            fn get_represented_type_info(&self) -> Option<&'static TypeInfo> {

                Some(<Self as Typed>::type_info())

            }


            #[inline]

            fn into_partial_reflect(self: Box<Self>) -> Box<dyn PartialReflect> {

                self

            }


            fn as_partial_reflect(&self) -> &dyn PartialReflect {

                self

            }


            fn as_partial_reflect_mut(&mut self) -> &mut dyn PartialReflect {

                self

            }


            #[inline]

            fn try_into_reflect(

                self: Box<Self>,

            ) -> Result<Box<dyn Reflect>, Box<dyn PartialReflect>> {

                Ok(self)

            }


            fn try_as_reflect(&self) -> Option<&dyn Reflect> {

                Some(self)

            }


            fn try_as_reflect_mut(&mut self) -> Option<&mut dyn Reflect> {

                Some(self)

            }


            fn apply(&mut self, value: &dyn PartialReflect) {

                set_apply(self, value);

            }


            fn try_apply(&mut self, value: &dyn PartialReflect) -> Result<(), ApplyError> {

                set_try_apply(self, value)

            }


            fn reflect_kind(&self) -> ReflectKind {

                ReflectKind::Set

            }


            fn reflect_ref(&self) -> ReflectRef<'_> {

                ReflectRef::Set(self)

            }


            fn reflect_mut(&mut self) -> ReflectMut<'_> {

                ReflectMut::Set(self)

            }


            fn reflect_owned(self: Box<Self>) -> ReflectOwned {

                ReflectOwned::Set(self)

            }


            fn reflect_clone(&self) -> Result<Box<dyn Reflect>, ReflectCloneError> {

                let mut set = Self::with_capacity_and_hasher(self.len(), S::default());

                for value in self.iter() {

                    let value = value.reflect_clone()?.take().map_err(|_| {

                        ReflectCloneError::FailedDowncast {

                            expected: Cow::Borrowed(<V as TypePath>::type_path()),

                            received: Cow::Owned(value.reflect_type_path().to_string()),

                        }

                    })?;

                    set.insert(value);

                }


                Ok(Box::new(set))

            }


            fn reflect_partial_eq(&self, value: &dyn PartialReflect) -> Option<bool> {

                set_partial_eq(self, value)

            }

        }


        impl<V, S> Typed for $ty

        where

            V: FromReflect + TypePath + GetTypeRegistration + Eq + Hash,

            S: TypePath + BuildHasher + Default + Send + Sync,

        {

            fn type_info() -> &'static TypeInfo {

                static CELL: GenericTypeInfoCell = GenericTypeInfoCell::new();

                CELL.get_or_insert::<Self, _>(|| {

                    TypeInfo::Set(

                        SetInfo::new::<Self, V>().with_generics(Generics::from_iter([

                            TypeParamInfo::new::<V>("V")

                        ]))

                    )

                })

            }

        }


        impl<V, S> GetTypeRegistration for $ty

        where

            V: FromReflect + TypePath + GetTypeRegistration + Eq + Hash,

            S: TypePath + BuildHasher + Default + Send + Sync + Default,

        {

            fn get_type_registration() -> TypeRegistration {

                let mut registration = TypeRegistration::of::<Self>();

                registration.insert::<ReflectFromPtr>(FromType::<Self>::from_type());

                registration.insert::<ReflectFromReflect>(FromType::<Self>::from_type());

                registration

            }


            fn register_type_dependencies(registry: &mut TypeRegistry) {

                registry.register::<V>();

            }

        }


        impl_full_reflect!(

            <V, S> for $ty

            where

                V: FromReflect + TypePath + GetTypeRegistration + Eq + Hash,

                S: TypePath + BuildHasher + Default + Send + Sync,

        );


        impl<V, S> FromReflect for $ty

        where

            V: FromReflect + TypePath + GetTypeRegistration + Eq + Hash,

            S: TypePath + BuildHasher + Default + Send + Sync,

        {

            fn from_reflect(reflect: &dyn PartialReflect) -> Option<Self> {

                let ref_set = reflect.reflect_ref().as_set().ok()?;


                let mut new_set = Self::with_capacity_and_hasher(ref_set.len(), S::default());


                for value in ref_set.iter() {

                    let new_value = V::from_reflect(value)?;

                    new_set.insert(new_value);

                }


                Some(new_set)

            }

        }

    };

}
#[cfg(feature = "functions")]
macro_rules! impl_function_traits {

    (

        $ty: ty

        $(;

            <

                $($T: ident $(: $T1: tt $(+ $T2: tt)*)?),*

            >

        )?

        $(

            [

                $(const $N: ident : $size: ident),*

            ]

        )?

        $(

            where

                $($U: ty $(: $U1: tt $(+ $U2: tt)*)?),*

        )?

    ) => {

        impl_get_ownership!(

            $ty

            $(;

                <

                    $($T $(: $T1 $(+ $T2)*)?),*

                >

            )?

            $(

                [

                    $(const $N : $size),*

                ]

            )?

            $(

                where

                    $($U $(: $U1 $(+ $U2)*)?),*

            )?

        );

        impl_from_arg!(

            $ty

            $(;

                <

                    $($T $(: $T1 $(+ $T2)*)?),*

                >

            )?

            $(

                [

                    $(const $N : $size),*

                ]

            )?

            $(

                where

                    $($U $(: $U1 $(+ $U2)*)?),*

            )?

        );

        impl_into_return!(

            $ty

            $(;

                <

                    $($T $(: $T1 $(+ $T2)*)?),*

                >

            )?

            $(

                [

                    $(const $N : $size),*

                ]

            )?

            $(

                where

                    $($U $(: $U1 $(+ $U2)*)?),*

            )?

        );

    };

}

#[cfg(feature = "functions")]
macro_rules! impl_from_arg {

    (

        $ty: ty

        $(;

            <

                $($T: ident $(: $T1: tt $(+ $T2: tt)*)?),*

            >

        )?

        $(

            [

                $(const $N: ident : $size: ident),*

            ]

        )?

        $(

            where

                $($U: ty $(: $U1: tt $(+ $U2: tt)*)?),*

        )?

    ) => {

        impl <

            $($($T $(: $T1 $(+ $T2)*)?),*)?

            $(, $(const $N : $size),*)?

        > FromArg for $ty

        $(

            where

                $($U $(: $U1 $(+ $U2)*)?),*

        )?

        {

            type This<'from_arg> = $ty;

            fn from_arg(arg: Arg<'_>) -> Result<Self::This<'_>, ArgError> {

                arg.take_owned()

            }

        }


        impl <

            $($($T $(: $T1 $(+ $T2)*)?),*)?

            $(, $(const $N : $size),*)?

        > FromArg for &'static $ty

        $(

            where

                $($U $(: $U1 $(+ $U2)*)?),*

        )?

        {

            type This<'from_arg> = &'from_arg $ty;

            fn from_arg(arg: Arg<'_>) -> Result<Self::This<'_>, ArgError> {

                arg.take_ref()

            }

        }


        impl <

            $($($T $(: $T1 $(+ $T2)*)?),*)?

            $(, $(const $N : $size),*)?

        > FromArg for &'static mut $ty

        $(

            where

                $($U $(: $U1 $(+ $U2)*)?),*

        )?

        {

            type This<'from_arg> = &'from_arg mut $ty;

            fn from_arg(arg: Arg<'_>) -> Result<Self::This<'_>, ArgError> {

                arg.take_mut()

            }

        }

    };

}

#[cfg(feature = "functions")]
macro_rules! impl_get_ownership {

    (

        $ty: ty

        $(;

            <

                $($T: ident $(: $T1: tt $(+ $T2: tt)*)?),*

            >

        )?

        $(

            [

                $(const $N: ident : $size: ident),*

            ]

        )?

        $(

            where

                $($U: ty $(: $U1: tt $(+ $U2: tt)*)?),*

        )?

    ) => {

        impl <

            $($($T $(: $T1 $(+ $T2)*)?),*)?

            $(, $(const $N : $size),*)?

        > GetOwnership for $ty

        $(

            where

                $($U $(: $U1 $(+ $U2)*)?),*

        )?

        {

            fn ownership() -> Ownership {

                Ownership::Owned

            }

        }


        impl <

            $($($T $(: $T1 $(+ $T2)*)?),*)?

            $(, $(const $N : $size),*)?

        > GetOwnership for &'_ $ty

        $(

            where

                $($U $(: $U1 $(+ $U2)*)?),*

        )?

        {

            fn ownership() -> Ownership {

                Ownership::Ref

            }

        }


        impl <

            $($($T $(: $T1 $(+ $T2)*)?),*)?

            $(, $(const $N : $size),*)?

        > GetOwnership for &'_ mut $ty

        $(

            where

                $($U $(: $U1 $(+ $U2)*)?),*

        )?

        {

            fn ownership() -> Ownership {

                Ownership::Mut

            }

        }

    };

}

#[cfg(feature = "functions")]
macro_rules! impl_into_return {

    (

        $ty: ty

        $(;

            <

                $($T: ident $(: $T1: tt $(+ $T2: tt)*)?),*

            >

        )?

        $(

            [

                $(const $N: ident : $size: ident),*

            ]

        )?

        $(

            where

                $($U: ty $(: $U1: tt $(+ $U2: tt)*)?),*

        )?

    ) => {

        impl <

            $($($T $(: $T1 $(+ $T2)*)?),*)?

            $(, $(const $N : $size),*)?

        > IntoReturn for $ty

        $(

            where

                $($U $(: $U1 $(+ $U2)*)?),*

        )?

        {

            fn into_return<'into_return>(self) -> Return<'into_return> where Self: 'into_return {

                Owned(Box::new(self))

            }

        }


        impl <

            $($($T $(: $T1 $(+ $T2)*)?),*)?

            $(, $(const $N : $size),*)?

        > IntoReturn for &'static $ty

        $(

            where

                $($U $(: $U1 $(+ $U2)*)?),*

        )?

        {

            fn into_return<'into_return>(self) -> Return<'into_return> where Self: 'into_return {

                Return::Ref(self)

            }

        }


        impl <

            $($($T $(: $T1 $(+ $T2)*)?),*)?

            $(, $(const $N : $size),*)?

        > IntoReturn for &'static mut $ty

        $(

            where

                $($U $(: $U1 $(+ $U2)*)?),*

        )?

        {

            fn into_return<'into_return>(self) -> Return<'into_return> where Self: 'into_return {

                Return::Mut(self)

            }

        }

    };

}


#[cfg(feature = "functions")]
pub(crate) use impl_into_return;
#[cfg(feature = "functions")]
pub(crate) use impl_get_ownership;
#[cfg(feature = "functions")]
pub(crate) use impl_from_arg;
#[cfg(feature = "functions")]
pub(crate) use impl_function_traits;
pub(crate) use impl_full_reflect;
pub(crate) use impl_reflect_for_hashmap;
pub(crate) use impl_reflect_for_hashset;