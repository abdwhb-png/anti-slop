<?php
declare(strict_types=1);
// Qualification only. Import the pinned upstream SDK; do not copy its protocol.
$sdkRoot=$argv[1];
require $argv[2];
spl_autoload_register(static function(string $name) use ($sdkRoot): void {
    if (str_starts_with($name, 'Mago\\')) {
        $file=$sdkRoot.'/'.str_replace('\\','/',substr($name,5)).'.php';
        if(is_file($file)) require $file;
    }
});
use Mago\Sdk\Analyzer\{Plugin,PluginDefinition,PluginRegistry,AfterAnalysisHook,AfterAnalysisContext};
use Mago\Sdk\Analyzer\Metadata\MemberIdentifier;
use Mago\Sdk\{Extension,Worker};

final class EvidenceProbe implements Plugin, AfterAnalysisHook {
    public function __construct(private string $output) {}
    public function getDefinition(): PluginDefinition { return new PluginDefinition('qualification/evidence','Evidence probe','Extract native evidence for qualification'); }
    public function register(PluginRegistry $registry): void { $registry->registerAfterAnalysisHook($this); }
    public function afterAnalysis(AfterAnalysisContext $context): void {
        $files=[];
        foreach ($context->analysis->files as $analysis) {
            $source=$analysis->getSourceFile();
            $types=[];
            foreach ($analysis->getAllExpressionTypes() as $expression) {
                $types[]=['start'=>$expression->span->start,'end'=>$expression->span->end,'type'=>(string)$expression->type,'atomicKinds'=>array_map(static fn(object $type): string => get_class($type),$expression->type->atomicTypes)];
            }
            $files[]=['file'=>$analysis->file,'source'=>$source->contents,'expressionTypes'=>$types,'references'=>$analysis->references];
        }
        $class=$context->codebase->getClass('Fixture\\MemoryRepository');
        $interface=$context->codebase->getInterface('Fixture\\Repository');
        $member=new MemberIdentifier('Fixture\\Repository','load');
        $result=['analyzer'=>'mago','version'=>'1.48.1','issueCount'=>$context->analysis->issueCount,'files'=>$files,'class'=>$class,'interface'=>$interface,'incoming'=>$context->analysis->references->getReferencesTo($member),'outgoing'=>$context->analysis->references->getReferencesFrom('Fixture\\serve'),'referencesSummary'=>$context->analysis->references->summary];
        $normalize=function(mixed $value) use (&$normalize): mixed {
            if($value instanceof BackedEnum) return $value->value;
            if($value instanceof UnitEnum) return $value->name;
            if(is_object($value)) $value=get_object_vars($value);
            return is_array($value)?array_map($normalize,$value):$value;
        };
        $json=json_encode($normalize($result),JSON_THROW_ON_ERROR|JSON_UNESCAPED_SLASHES);
        if(file_put_contents($this->output,$json."\n",FILE_APPEND|LOCK_EX)===false) throw new RuntimeException('Cannot retain evidence');
    }
}
(new Worker(new Extension(identifier:'qualification/evidence',name:'Qualification',version:'0.1.0',analyzerPlugins:[new EvidenceProbe($argv[3])])))->run();
