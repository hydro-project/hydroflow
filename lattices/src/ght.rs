use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use sealed::sealed;
use variadics::{
    var_args, var_type, PartialEqVariadic, RefVariadic, Split, SplitBySuffix, VariadicExt,
};

use crate::ght_lattice::DeepJoinLatticeBimorphism;

/// GeneralizedHashTrieNode trait
#[sealed]
pub trait GeneralizedHashTrieNode: Default {
    /// Schema variadic: the schema of the relation stored in this trie.
    /// This type is the same in all nodes of the trie.
    type Schema: VariadicExt + Eq + Hash + Clone + SplitBySuffix<Self::SuffixSchema>;
    /// The prefix of columns in Schema that the trie is keyed on
    /// This type is the same in all nodes of the trie.
    type KeyType: VariadicExt + Eq + Hash + Clone;
    /// The suffix of columns in Schema that are not part of the trie keys
    /// This type is the same in all nodes of the trie.
    type ValType: VariadicExt + Eq + Hash + Clone;

    /// SuffixSchema variadic: the suffix of the schema *from this node of the trie
    /// downward*. The first entry in this variadic is of type Head.
    type SuffixSchema: VariadicExt + Eq + Hash + Clone;

    // Head is the first field in SuffixSchema
    type Head: Eq + Hash + Clone;

    /// Create a new Ght from the iterator.
    fn new_from(input: impl IntoIterator<Item = Self::Schema>) -> Self;

    /// Report the height of the tree if its not empty. This is the length of a root to leaf path -1.
    /// E.g. if we have GhtInner<GhtInner<GhtLeaf...>> the height is 2
    fn height(&self) -> usize {
        Self::static_height()
    }

    /// Return the height of this node in the GhT. Leaf = 0.
    fn static_height() -> usize;

    /// report whether node is a leaf node; else an inner node
    fn is_leaf(&self) -> bool;

    /// Inserts items into the hash trie.
    fn insert(&mut self, row: Self::Schema) -> bool;

    /// Returns `true` if the (entire) row is found in the trie, `false` otherwise.
    /// See `get()` below to look just for "head" keys in this node
    fn contains<'a>(&'a self, row: <Self::Schema as VariadicExt>::AsRefVar<'a>) -> bool;

    /// Iterate through the (entire) rows stored in this HashTrie.
    /// Returns Variadics, not tuples.
    fn recursive_iter(&self) -> impl Iterator<Item = <Self::Schema as VariadicExt>::AsRefVar<'_>>;

    // /// Iterate through the (entire) rows stored in this HashTrie, but with the leaf
    // /// values stubbed out ((), ()).
    // fn recursive_iter_keys(
    //     &self,
    // ) -> impl Iterator<Item = <Self::Schema as VariadicExt>::AsRefVar<'_>>;

    /// Bimorphism for joining on full tuple keys (all GhtInner keys) in the trie
    type DeepJoin<Other>
    where
        Other: GeneralizedHashTrieNode,
        (Self, Other): DeepJoinLatticeBimorphism;

    // /// For Inner nodes only, this is the type of the Child node
    // type ChildNode: GeneralizedHashTrieNode;

    // /// Cast as Some<&GhtInner> if this is an inner node, else return None
    // fn cast_as_inner(&self) -> Option<&Self>;

    // /// Cast as Some<&mut GhtInner> if this is an inner node, else return None
    // fn cast_as_inner_mut(&mut self) -> Option<&mut Self>;

    /// return the leaf in the trie that contains this row
    fn find_containing_leaf(
        &self,
        row: <Self::Schema as VariadicExt>::AsRefVar<'_>,
    ) -> Option<&'_ GhtLeaf<Self::Schema, Self::ValType>>;
}

pub trait GhtKeyTrait<Head, Schema> {
    fn head(self) -> Option<Head>;
    fn schema(self) -> Option<Schema>;
}

impl<Head, Schema> GhtKeyTrait<Head, Schema> for GhtKey<Head, Schema> {
    fn head(self) -> Option<Head> {
        if let GhtKey::Head(head) = self {
            Some(head)
        } else {
            None
        }
    }
    fn schema(self) -> Option<Schema> {
        if let GhtKey::Schema(head) = self {
            Some(head)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub enum GhtKey<Head, Schema> {
    Head(Head),
    Schema(Schema),
}

impl<Head, Schema> PartialEq for GhtKey<Head, Schema>
where
    Head: Eq,
    Schema: Eq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (GhtKey::Head(x), GhtKey::Head(y)) => x == y,
            (GhtKey::Schema(x), GhtKey::Schema(y)) => x == y,
            _ => false,
        }
    }
}

impl<Head, Schema> Eq for GhtKey<Head, Schema>
where
    Head: Eq,
    Schema: Eq,
{
}

impl<Head, Schema> Hash for GhtKey<Head, Schema>
where
    Head: Hash,
    Schema: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            GhtKey::Head(x) => x.hash(state),
            GhtKey::Schema(x) => x.hash(state),
        }
    }
}

/// internal node of a HashTrie
#[derive(Debug, Clone)]
pub struct GhtInner<Head, Node>
where
    Head: Clone,
    Node: GeneralizedHashTrieNode,
{
    pub(crate) children: HashMap<Head, Node>,
    // pub(crate) _leaf: std::marker::PhantomData<Leaf>,
}
impl<Head, Node: GeneralizedHashTrieNode> Default for GhtInner<Head, Node>
where
    Head: Clone,
    Node: GeneralizedHashTrieNode,
{
    fn default() -> Self {
        let children = Default::default();
        Self {
            children,
            // _leaf: Default::default(),
        }
    }
}
#[sealed]
impl<Head, Node> GeneralizedHashTrieNode for GhtInner<Head, Node>
where
    Head: 'static + Hash + Eq + Clone,
    Node: 'static + GeneralizedHashTrieNode,
    Node::Schema: SplitBySuffix<var_type!(Head, ...Node::SuffixSchema)>,
{
    type Schema = Node::Schema;
    type SuffixSchema = var_type!(Head, ...Node::SuffixSchema);
    type ValType = Node::ValType;
    type KeyType = Node::KeyType;
    type Head = Head;

    fn new_from(input: impl IntoIterator<Item = Self::Schema>) -> Self {
        let mut retval: Self = Default::default();
        let mut result = true;
        for row in input {
            // let (_prefix, suffix) = row.clone().split();
            result = result && retval.insert(row);
        }
        retval
    }

    fn static_height() -> usize {
        Node::static_height() + 1
    }

    fn is_leaf(&self) -> bool {
        false
    }

    fn insert(&mut self, row: Self::Schema) -> bool {
        // TODO(mingwei): clones entire row...
        let (_prefix, var_args!(head, ..._rest)) = row.clone().split_by_suffix();
        self.children.entry(head).or_default().insert(row)
    }

    fn contains<'a>(&'a self, row: <Self::Schema as VariadicExt>::AsRefVar<'a>) -> bool {
        //  as SplitBySuffix<Self::SuffixSchema>>
        let (_prefix, var_args!(head, ..._rest)) = Self::Schema::split_by_suffix_ref(row);
        if let Some(node) = self.children.get(head) {
            node.contains(row)
        } else {
            false
        }
    }

    fn recursive_iter(&self) -> impl Iterator<Item = <Self::Schema as VariadicExt>::AsRefVar<'_>> {
        self.children
            .iter()
            .flat_map(|(_k, vs)| vs.recursive_iter())
    }

    // fn recursive_iter_keys(
    //     &self,
    // ) -> impl Iterator<Item = <Self::Schema as VariadicExt>::AsRefVar<'_>> {
    //     self.children
    //         .iter()
    //         .flat_map(|(_k, vs)| vs.recursive_iter_keys().map(move |v| v))
    // }

    type DeepJoin<Other> = <(Self, Other) as DeepJoinLatticeBimorphism>::DeepJoinLatticeBimorphism
    where
        Other: GeneralizedHashTrieNode,
        (Self, Other): DeepJoinLatticeBimorphism;

    fn find_containing_leaf(
        &self,
        row: <Self::Schema as VariadicExt>::AsRefVar<'_>,
    ) -> Option<&'_ GhtLeaf<Self::Schema, Self::ValType>> {
        let (_prefix, var_args!(head, ..._rest)) = Self::Schema::split_by_suffix_ref(row);
        self.children
            .get(head)
            .and_then(|child| child.find_containing_leaf(row))
    }
}

impl<Head, Node> FromIterator<Node::Schema> for GhtInner<Head, Node>
where
    Head: 'static + Hash + Eq + Clone,
    Node: 'static + GeneralizedHashTrieNode + Clone,
    Node::Schema: SplitBySuffix<var_type!(Head, ...Node::SuffixSchema)>,
{
    fn from_iter<Iter: IntoIterator<Item = Node::Schema>>(iter: Iter) -> Self {
        let mut out = Self::default();
        for row in iter {
            out.insert(row);
        }
        out
    }
}

impl<Schema, ValType> FromIterator<Schema> for GhtLeaf<Schema, ValType>
where
    Schema: Eq + Hash,
{
    fn from_iter<Iter: IntoIterator<Item = Schema>>(iter: Iter) -> Self {
        let elements = iter.into_iter().collect();
        Self {
            elements,
            forced: false,
            _suffix_schema: PhantomData,
        }
    }
}

/// leaf node of a HashTrie
#[derive(Debug, PartialEq, Eq, Clone)]
// #[repr(transparent)]
pub struct GhtLeaf<Schema, ValType>
where
    Schema: Eq + Hash,
{
    pub(crate) elements: HashSet<Schema>,
    pub(crate) forced: bool,
    pub(crate) _suffix_schema: PhantomData<ValType>,
}
impl<Schema, ValType> Default for GhtLeaf<Schema, ValType>
where
    Schema: Eq + Hash,
{
    fn default() -> Self {
        let elements = Default::default();
        Self {
            elements,
            forced: false,
            _suffix_schema: PhantomData,
        }
    }
}

#[sealed]
impl<Schema, ValHead, ValRest> GeneralizedHashTrieNode
    for GhtLeaf<Schema, var_type!(ValHead, ...ValRest)>
where
    Schema: 'static
        + Eq
        + VariadicExt
        + Hash
        + Clone
        + SplitBySuffix<var_type!(ValHead, ...ValRest)>
        + PartialEqVariadic,
    ValHead: Clone + Eq + Hash,
    var_type!(ValHead, ...ValRest): Clone + Eq + Hash + PartialEqVariadic,
    <Schema as SplitBySuffix<var_type!(ValHead, ...ValRest)>>::Prefix: Eq + Hash + Clone,
    // for<'a> Schema::AsRefVar<'a>: PartialEq,
{
    type Schema = Schema;
    type SuffixSchema = var_type!(ValHead, ...ValRest);
    type ValType = var_type!(ValHead, ...ValRest);
    type KeyType = <Schema as SplitBySuffix<var_type!(ValHead, ...ValRest)>>::Prefix;
    type Head = ValHead;

    fn new_from(input: impl IntoIterator<Item = Self::Schema>) -> Self {
        let mut retval: Self = Default::default();
        for i in input {
            retval.insert(i);
        }
        retval
    }

    fn static_height() -> usize {
        0
    }

    fn is_leaf(&self) -> bool {
        true
    }

    fn insert(&mut self, row: Self::Schema) -> bool {
        self.elements.insert(row)
    }

    fn contains<'a>(&'a self, row: <Self::Schema as VariadicExt>::AsRefVar<'a>) -> bool {
        self.elements
            .iter()
            .any(|r| Schema::eq_ref(r.as_ref_var(), row))
    }

    fn recursive_iter(&self) -> impl Iterator<Item = <Self::Schema as VariadicExt>::AsRefVar<'_>> {
        self.elements.iter().map(Schema::as_ref_var)
    }

    type DeepJoin<Other> = <(Self, Other) as DeepJoinLatticeBimorphism>::DeepJoinLatticeBimorphism
    where
        Other: GeneralizedHashTrieNode,
        (Self, Other): DeepJoinLatticeBimorphism;

    fn find_containing_leaf(
        &self,
        row: <Self::Schema as VariadicExt>::AsRefVar<'_>,
    ) -> Option<&'_ GhtLeaf<<Self as GeneralizedHashTrieNode>::Schema, Self::ValType>> {
        // TODO(mingwei): actually use the hash set as a hash set
        if self.elements.iter().any(|x| {
            // let (_prefix, suffix) = Schema::split_by_suffix_ref(x.as_ref_var());
            <Schema as PartialEqVariadic>::eq_ref(row, x.as_ref_var())
        }) {
            Some(self)
        } else {
            None
        }
    }
}

// XXX JMH THIS IS UGLY. MOVE Head, KeyType, ValType, SuffixSchema to another trait
#[sealed]
impl<Schema> GeneralizedHashTrieNode for GhtLeaf<Schema, ()>
where
    Schema: 'static
        + Eq
        + VariadicExt
        + Hash
        + Clone
        // + SplitBySuffix<var_type!(ValHead, ...ValRest)>
        + PartialEqVariadic,
    // ValHead: Clone + Eq + Hash,
    // var_type!(ValHead, ...ValRest): Clone + Eq + Hash + PartialEqVariadic,
    // <Schema as SplitBySuffix<var_type!(ValHead, ...ValRest)>>::Prefix: Eq + Hash + Clone,
    // for<'a> Schema::AsRefVar<'a>: PartialEq,
{
    type Schema = Schema;
    type SuffixSchema = (); // var_type!(ValHead, ...ValRest);
    type ValType = (); // var_type!(ValHead, ...ValRest);
    type KeyType = Schema; //<Schema as SplitBySuffix<var_type!(ValHead, ...ValRest)>>::Prefix;
    type Head = ();

    fn new_from(input: impl IntoIterator<Item = Self::Schema>) -> Self {
        let mut retval: Self = Default::default();
        for i in input {
            retval.insert(i);
        }
        retval
    }

    fn static_height() -> usize {
        0
    }

    fn is_leaf(&self) -> bool {
        true
    }

    fn insert(&mut self, row: Self::Schema) -> bool {
        self.elements.insert(row)
    }

    fn contains<'a>(&'a self, row: <Self::Schema as VariadicExt>::AsRefVar<'a>) -> bool {
        self.elements
            .iter()
            .any(|r| Schema::eq_ref(r.as_ref_var(), row))
    }

    fn recursive_iter(&self) -> impl Iterator<Item = <Self::Schema as VariadicExt>::AsRefVar<'_>> {
        self.elements.iter().map(Schema::as_ref_var)
    }

    type DeepJoin<Other> = <(Self, Other) as DeepJoinLatticeBimorphism>::DeepJoinLatticeBimorphism
    where
        Other: GeneralizedHashTrieNode,
        (Self, Other): DeepJoinLatticeBimorphism;

    fn find_containing_leaf(
        &self,
        row: <Self::Schema as VariadicExt>::AsRefVar<'_>,
    ) -> Option<&'_ GhtLeaf<<Self as GeneralizedHashTrieNode>::Schema, Self::ValType>> {
        // TODO(mingwei): actually use the hash set as a hash set
        if self.elements.iter().any(|x| {
            // let (_prefix, suffix) = Schema::split_by_suffix_ref(x.as_ref_var());
            <Schema as PartialEqVariadic>::eq_ref(row, x.as_ref_var())
        }) {
            Some(self)
        } else {
            None
        }
    }
}

/// A trait for the get method, which works differently on leaves than internal nodes
pub trait GhtGet: GeneralizedHashTrieNode {
    /// Type returned by [`Self::get`].
    type Get: GeneralizedHashTrieNode<Schema = Self::Schema>;

    /// On an Inner node, retrieves the value (child) associated with the given "head" key.
    /// returns an `Option` containing a reference to the value if found, or `None` if not found.
    /// On a Leaf node, returns None.
    fn get<'a>(&'a self, head: &GhtKey<Self::Head, Self::Schema>) -> Option<&'a Self::Get>;

    fn get_mut<'a>(
        &'a mut self,
        head: &GhtKey<Self::Head, Self::Schema>,
    ) -> Option<&'a mut Self::Get>;

    // /// The type of items returned by iter
    // type IterKey: Eq + Hash;
    /// Iterator for the "head" keys (from inner nodes) or elements (from leaf nodes).
    fn iter(&self) -> impl Iterator<Item = GhtKey<Self::Head, Self::Schema>>;
}

impl<Head, Node> GhtGet for GhtInner<Head, Node>
where
    Head: 'static + Eq + Hash + Clone,
    Node: 'static + GeneralizedHashTrieNode,
    Node::Schema: SplitBySuffix<var_type!(Head, ...Node::SuffixSchema)>,
{
    /// Type returned by [`Self::get`].
    type Get = Node;

    /// On an Inner node, retrieves the value (child) associated with the given "head" key.
    /// returns an `Option` containing a reference to the value if found, or `None` if not found.
    /// On a Leaf node, returns None.
    fn get<'a>(&'a self, head: &GhtKey<Self::Head, Self::Schema>) -> Option<&'a Self::Get> {
        if let GhtKey::Head(the_head) = head {
            self.children.get(the_head)
        } else {
            None
        }
    }

    fn get_mut<'a>(
        &'a mut self,
        head: &GhtKey<Self::Head, Self::Schema>,
    ) -> Option<&'a mut Self::Get> {
        if let GhtKey::Head(the_head) = head {
            self.children.get_mut(the_head)
        } else {
            None
        }
    }

    // /// The type of items returned by iter
    // type IterKey = Self::Head;
    fn iter(&self) -> impl Iterator<Item = GhtKey<Self::Head, Self::Schema>> {
        self.children.keys().map(|k| GhtKey::Head(k.clone()))
    }
}
impl<Schema, ValType> GhtGet for GhtLeaf<Schema, ValType>
where
    Schema: 'static + Eq + Hash + Clone + PartialEqVariadic + SplitBySuffix<ValType>,
    ValType: Eq + Hash + Clone + PartialEqVariadic,
    <Schema as SplitBySuffix<ValType>>::Prefix: Eq + Hash + Clone,
    GhtLeaf<Schema, ValType>: GeneralizedHashTrieNode<Schema = Schema>,
{
    /// Type returned by [`Self::get`].
    type Get = GhtLeaf<Schema, ValType>;

    /// On an Inner node, retrieves the value (child) associated with the given "head" key.
    /// returns an `Option` containing a reference to the value if found, or `None` if not found.
    /// On a Leaf node, returns None.
    fn get<'a>(&'a self, _head: &GhtKey<Self::Head, Self::Schema>) -> Option<&'a Self::Get> {
        None
    }
    fn get_mut<'a>(
        &'a mut self,
        _head: &GhtKey<Self::Head, Self::Schema>,
    ) -> Option<&'a mut Self::Get> {
        None
    }

    // /// The type of items returned by iter
    // type IterKey = Self::Schema;
    fn iter(&self) -> impl Iterator<Item = GhtKey<Self::Head, Self::Schema>> {
        self.elements.iter().map(|e| GhtKey::Schema(e.clone()))
    }
}

/// Trait for taking a matching leaf out of a trie, as part of the COLT force
#[sealed]
pub trait GhtTakeLeaf: GeneralizedHashTrieNode {
    /// take a matching leaf out of a trie, as part of the COLT force
    fn take_containing_leaf(
        &mut self,
        row: <Self::Schema as VariadicExt>::AsRefVar<'_>,
    ) -> Option<GhtLeaf<Self::Schema, Self::ValType>>;

    /// Merges a leaf into the hash trie
    fn merge_leaf(
        &mut self,
        row: <Self::Schema as VariadicExt>::AsRefVar<'_>,
        leaf: GhtLeaf<Self::Schema, Self::ValType>,
    ) -> bool;
}

#[sealed]
impl<Head, Head2, GrandNode> GhtTakeLeaf for GhtInner<Head, GhtInner<Head2, GrandNode>>
where
    Head: 'static + Hash + Eq + Clone,
    Head2: 'static + Hash + Eq + Clone,
    GrandNode: 'static + GeneralizedHashTrieNode,
    GhtInner<Head2, GrandNode>: GhtTakeLeaf,
    <GhtInner<Head2, GrandNode> as GeneralizedHashTrieNode>::Schema: SplitBySuffix<
        var_type!(Head,  ...<GhtInner<Head2, GrandNode> as GeneralizedHashTrieNode>::SuffixSchema),
    >,
{
    fn take_containing_leaf(
        &mut self,
        row: <<GhtInner<Head2, GrandNode> as GeneralizedHashTrieNode>::Schema as VariadicExt>::AsRefVar<'_>,
    ) -> Option<
        GhtLeaf<
            <GhtInner<Head2, GrandNode> as GeneralizedHashTrieNode>::Schema,
            <GhtInner<Head2, GrandNode> as GeneralizedHashTrieNode>::ValType,
        >,
    > {
        let (_prefix, var_args!(head, ..._rest)) = Self::Schema::split_by_suffix_ref(row);
        self.children
            .get_mut(head)
            .and_then(|child| child.take_containing_leaf(row))
    }

    fn merge_leaf(
        &mut self,
        row: <Self::Schema as VariadicExt>::AsRefVar<'_>,
        leaf: GhtLeaf<Self::Schema, Self::ValType>,
    ) -> bool {
        // TODO(mingwei): clones head...
        let (_prefix, var_args!(head, ..._rest)) = Self::Schema::split_by_suffix_ref(row);
        let retval = self
            .children
            .entry(head.clone())
            .or_default()
            .merge_leaf(row, leaf);
        // self.height = self.get(head).and_then(|n| n.old_height).map(|h| h + 1);
        retval
    }
}

#[sealed]
impl<Head, Schema, ValType> GhtTakeLeaf for GhtInner<Head, GhtLeaf<Schema, ValType>>
where
    Schema: 'static
        + Hash
        + Clone
        + Eq
        + PartialEqVariadic
        + SplitBySuffix<ValType>
        + SplitBySuffix<var_type!(Head, ...ValType)>,
    ValType: 'static + Hash + Clone + Eq + VariadicExt + PartialEqVariadic,
    Head: 'static + Hash + Eq + Clone,
    <Schema as SplitBySuffix<ValType>>::Prefix: Eq + Hash + Clone,
    // <Self as GeneralizedHashTrieNode>::Schema:
    //     SplitBySuffix<<Self as GeneralizedHashTrieNode>::SuffixSchema>,
    GhtLeaf<Schema, ValType>: GeneralizedHashTrieNode<Schema = Schema, ValType = ValType>,
    // Schema: SplitBySuffix<<Self as GeneralizedHashTrieNode>::SuffixSchema>,
    Schema: SplitBySuffix<(
        Head,
        <GhtLeaf<Schema, ValType> as GeneralizedHashTrieNode>::SuffixSchema,
    )>,
{
    fn take_containing_leaf(
        &mut self,
        row: <Self::Schema as VariadicExt>::AsRefVar<'_>,
    ) -> Option<GhtLeaf<Self::Schema, Self::ValType>> {
        let (_prefix, var_args!(head, ..._rest)) =
            <<Self as GeneralizedHashTrieNode>::Schema as SplitBySuffix<
                <Self as GeneralizedHashTrieNode>::SuffixSchema,
            >>::split_by_suffix_ref(row);
        let retval = self.children.remove(head);
        let new_child = GhtLeaf::<Schema, ValType> {
            elements: Default::default(),
            forced: true,
            _suffix_schema: Default::default(),
        };
        self.children.insert(head.clone(), new_child);
        retval
    }

    fn merge_leaf(
        &mut self,
        row: <Self::Schema as VariadicExt>::AsRefVar<'_>,
        leaf: GhtLeaf<Self::Schema, Self::ValType>,
    ) -> bool {
        let (_prefix, var_args!(head, ..._rest)) =
            <<Self as GeneralizedHashTrieNode>::Schema as SplitBySuffix<
                <Self as GeneralizedHashTrieNode>::SuffixSchema,
            >>::split_by_suffix_ref(row);
        if let Some(old_val) = self.children.insert(head.clone(), leaf) {
            self.children.insert(head.clone(), old_val);
            panic!();
            false
        } else {
            true
        }
    }
}

#[sealed]
/// iterators for HashTries based on a prefix search
pub trait GhtPrefixIter<KeyPrefix> {
    /// the schema output
    type Item: VariadicExt;
    /// given a prefix, return an iterator through the items below
    fn prefix_iter<'a>(
        &'a self,
        prefix: KeyPrefix,
    ) -> impl Iterator<Item = <Self::Item as VariadicExt>::AsRefVar<'a>>
    where
        Self::Item: 'a;
}

#[sealed]
impl<'k, Head, Node, PrefixRest> GhtPrefixIter<var_type!(&'k Head, ...PrefixRest)>
    for GhtInner<Head, Node>
where
    Head: Eq + Hash + Clone,
    Node: GeneralizedHashTrieNode + GhtPrefixIter<PrefixRest>,
{
    type Item = <Node as GhtPrefixIter<PrefixRest>>::Item;
    fn prefix_iter<'a>(
        &'a self,
        prefix: var_type!(&'k Head, ...PrefixRest),
    ) -> impl Iterator<Item = <Self::Item as VariadicExt>::AsRefVar<'a>>
    where
        Self::Item: 'a,
    {
        let var_args!(head, ...rest) = prefix;
        self.children
            .get(head)
            .map(|node| node.prefix_iter(rest))
            .into_iter()
            .flatten()
    }
}
#[sealed]
impl<Head, Node> GhtPrefixIter<var_type!()> for GhtInner<Head, Node>
where
    Self: GeneralizedHashTrieNode,
    Head: Eq + Hash + Clone,
    Node: GeneralizedHashTrieNode,
{
    type Item = <Self as GeneralizedHashTrieNode>::Schema;
    fn prefix_iter<'a>(
        &'a self,
        _prefix: var_type!(),
    ) -> impl Iterator<Item = <Self::Item as VariadicExt>::AsRefVar<'a>>
    where
        Self::Item: 'a,
    {
        self.recursive_iter()
    }
}

/// This case only splits HEAD and REST in order to prevent a conflict with the `HtPrefixIter<var_type!()>` impl.
/// If not for that, we could just use a single variadic type parameter.
#[sealed]
impl<KeyPrefixRef, Schema, ValType> GhtPrefixIter<KeyPrefixRef> for GhtLeaf<Schema, ValType>
where
    KeyPrefixRef: 'static + RefVariadic,
    Schema: 'static + VariadicExt + Hash + Eq + SplitBySuffix<ValType>,
    ValType: VariadicExt,
    // for<'a> SuffixSchema::AsRefVar<'a>: Split<KeyPrefixRef>,
    ValType: Split<KeyPrefixRef::UnRefVar>,
    KeyPrefixRef::UnRefVar: PartialEqVariadic,
{
    type Item = Schema;
    fn prefix_iter<'a>(
        &'a self,
        prefix: KeyPrefixRef,
    ) -> impl Iterator<Item = <Self::Item as VariadicExt>::AsRefVar<'a>>
    where
        Self::Item: 'a,
    {
        self.elements
            .iter()
            .map(Schema::as_ref_var)
            .filter(move |&row| {
                let (_row_prefix, row_mid_suffix) =
                    <Schema as SplitBySuffix<ValType>>::split_by_suffix_ref(row);
                let (row_mid, _row_suffix): (
                    <KeyPrefixRef::UnRefVar as VariadicExt>::AsRefVar<'_>,
                    _,
                ) = <ValType as Split<KeyPrefixRef::UnRefVar>>::split_ref(row_mid_suffix);
                <KeyPrefixRef::UnRefVar as PartialEqVariadic>::eq_ref(prefix.unref_ref(), row_mid)
            })
    }
}

/// Helper that does the heavy lifting for GhtType!
#[macro_export]
macro_rules! GhtTypeWithSchema {
    // Empty key & Val (Leaf)
    (() => () => $( $schema:ty ),+ ) => (
        $crate::ght::GhtLeaf::<$( $schema ),*,  ()  >
    );

    // Empty key (Leaf)
    (() => $( $z:ty ),* => $schema:ty ) => (
        $crate::ght::GhtLeaf::<$schema,  $crate::variadics::var_type!($( $z ),* ) >
    );

    // Singleton key & Empty val (Inner over Leaf)
    ($a:ty => () => $schema:ty ) => (
        $crate::ght::GhtInner::<$a, $crate::ght::GhtLeaf::<$schema, () >>
    );

    // Singleton key (Inner over Leaf)
    ($a:ty => $( $z:ty ),* => $schema:ty ) => (
        $crate::ght::GhtInner::<$a, $crate::ght::GhtLeaf::<$schema, $crate::variadics::var_type!($( $z ),*) >>
    );

    // Recursive case with empty val
    ($a:ty, $( $b:ty ),* => () => $schema:ty ) => (
        $crate::ght::GhtInner::<$a, $crate::GhtTypeWithSchema!($( $b ),* => () => $schema)>
    );

    // Recursive case.
    ($a:ty, $( $b:ty ),* => $( $z:ty ),* => $schema:ty ) => (
        $crate::ght::GhtInner::<$a, $crate::GhtTypeWithSchema!($( $b ),* => $( $z ),* => $schema)>
    );
}

/// Macro to construct a Ght node type from the constituent key and
/// dependent column types. You pass it:
///    - a list of key column types and dependent column type separated by a fat arrow,
///         a la (K1, K2, K3 => T1, T2, T3)
///
/// This macro generates a hierarchy of GHT node types where each key column is associated with an GhtInner
/// of the associated column type, and the remaining dependent columns are associated with a variadic HTleaf
/// a la var_expr!(T1, T2, T3)
#[macro_export]
macro_rules! GhtType {
    // Empty key
    (() => $( $z:ty ),* ) => (
        $crate::GhtTypeWithSchema!(() => $( $z ),* => $crate::variadics::var_type!($( $z ),* ))
    );

    // Recursive case empty val
    ($( $b:ty ),* => () ) => (
        $crate::GhtTypeWithSchema!($( $b ),* => () => $crate::variadics::var_type!($( $b ),*))
    );

    // Recursive case
    ($( $b:ty ),* => $( $z:ty ),*) => (
        $crate::GhtTypeWithSchema!($( $b ),* => $( $z ),* => $crate::variadics::var_type!($( $b ),*, $( $z ),*))
    );
}
